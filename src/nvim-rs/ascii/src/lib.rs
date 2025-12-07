//! ASCII character classification utilities for Neovim
//!
//! This crate provides pure functions for ASCII character classification,
//! exported via FFI for use in C code.

use std::ffi::c_int;

// ============================================================================
// ASCII Character Classification Functions
// These are pure functions with no external dependencies
// ============================================================================

/// Checks if `c` is a space or tab character (' ' or '\t').
///
/// # Arguments
/// * `c` - The character code to check
///
/// # Returns
/// 1 if the character is a space or tab, 0 otherwise
#[no_mangle]
pub extern "C" fn rs_ascii_iswhite(c: c_int) -> c_int {
    c_int::from(c == i32::from(b' ') || c == i32::from(b'\t'))
}

/// Checks if `c` is a space or tab character or NUL.
///
/// # Arguments
/// * `c` - The character code to check
///
/// # Returns
/// 1 if the character is space, tab, or NUL, 0 otherwise
#[no_mangle]
pub extern "C" fn rs_ascii_iswhite_or_nul(c: c_int) -> c_int {
    c_int::from(rs_ascii_iswhite(c) != 0 || c == 0)
}

/// Checks if `c` is a space, tab, newline, or NUL.
///
/// # Arguments
/// * `c` - The character code to check
///
/// # Returns
/// 1 if the character is space, tab, newline, or NUL, 0 otherwise
#[no_mangle]
pub extern "C" fn rs_ascii_iswhite_nl_or_nul(c: c_int) -> c_int {
    c_int::from(rs_ascii_iswhite(c) != 0 || c == i32::from(b'\n') || c == 0)
}

/// Check whether character is a decimal digit ('0'-'9').
///
/// This is locale-independent, unlike the standard library isdigit().
///
/// # Arguments
/// * `c` - The character code to check
///
/// # Returns
/// 1 if the character is a decimal digit, 0 otherwise
#[no_mangle]
pub extern "C" fn rs_ascii_isdigit(c: c_int) -> c_int {
    c_int::from(c >= i32::from(b'0') && c <= i32::from(b'9'))
}

/// Checks if `c` is a hexadecimal digit ('0'-'9', 'a'-'f', 'A'-'F').
///
/// # Arguments
/// * `c` - The character code to check
///
/// # Returns
/// 1 if the character is a hex digit, 0 otherwise
#[no_mangle]
pub extern "C" fn rs_ascii_isxdigit(c: c_int) -> c_int {
    c_int::from(
        (c >= i32::from(b'0') && c <= i32::from(b'9'))
            || (c >= i32::from(b'a') && c <= i32::from(b'f'))
            || (c >= i32::from(b'A') && c <= i32::from(b'F')),
    )
}

/// Checks if `c` is a binary digit ('0' or '1').
///
/// # Arguments
/// * `c` - The character code to check
///
/// # Returns
/// 1 if the character is a binary digit, 0 otherwise
#[no_mangle]
pub extern "C" fn rs_ascii_isbdigit(c: c_int) -> c_int {
    c_int::from(c == i32::from(b'0') || c == i32::from(b'1'))
}

/// Checks if `c` is an octal digit ('0'-'7').
///
/// # Arguments
/// * `c` - The character code to check
///
/// # Returns
/// 1 if the character is an octal digit, 0 otherwise
#[no_mangle]
pub extern "C" fn rs_ascii_isodigit(c: c_int) -> c_int {
    c_int::from(c >= i32::from(b'0') && c <= i32::from(b'7'))
}

/// Checks if `c` is a white-space character ('\f', '\n', '\r', '\t', '\v', or ' ').
///
/// # Arguments
/// * `c` - The character code to check
///
/// # Returns
/// 1 if the character is a white-space character, 0 otherwise
#[no_mangle]
pub extern "C" fn rs_ascii_isspace(c: c_int) -> c_int {
    // '\f' = 12, '\n' = 10, '\r' = 13, '\t' = 9, '\v' = 11, ' ' = 32
    // The range 9-13 covers \t, \n, \v, \f, \r
    c_int::from((c >= 9 && c <= 13) || c == i32::from(b' '))
}

/// Checks if `c` is an ASCII uppercase letter ('A'-'Z').
///
/// # Arguments
/// * `c` - The character code to check
///
/// # Returns
/// 1 if the character is uppercase, 0 otherwise
#[no_mangle]
pub extern "C" fn rs_ascii_isupper(c: c_int) -> c_int {
    c_int::from(c >= i32::from(b'A') && c <= i32::from(b'Z'))
}

/// Checks if `c` is an ASCII lowercase letter ('a'-'z').
///
/// # Arguments
/// * `c` - The character code to check
///
/// # Returns
/// 1 if the character is lowercase, 0 otherwise
#[no_mangle]
pub extern "C" fn rs_ascii_islower(c: c_int) -> c_int {
    c_int::from(c >= i32::from(b'a') && c <= i32::from(b'z'))
}

/// Checks if `c` is an ASCII alphabetic character ('A'-'Z' or 'a'-'z').
///
/// # Arguments
/// * `c` - The character code to check
///
/// # Returns
/// 1 if the character is alphabetic, 0 otherwise
#[no_mangle]
pub extern "C" fn rs_ascii_isalpha(c: c_int) -> c_int {
    c_int::from(rs_ascii_isupper(c) != 0 || rs_ascii_islower(c) != 0)
}

/// Checks if `c` is an ASCII alphanumeric character ('A'-'Z', 'a'-'z', '0'-'9').
///
/// # Arguments
/// * `c` - The character code to check
///
/// # Returns
/// 1 if the character is alphanumeric, 0 otherwise
#[no_mangle]
pub extern "C" fn rs_ascii_isalnum(c: c_int) -> c_int {
    c_int::from(rs_ascii_isalpha(c) != 0 || rs_ascii_isdigit(c) != 0)
}

/// Checks if `c` is an "identifier" character (alphanumeric or underscore).
///
/// # Arguments
/// * `c` - The character code to check
///
/// # Returns
/// 1 if the character is an identifier character, 0 otherwise
#[no_mangle]
pub extern "C" fn rs_ascii_isident(c: c_int) -> c_int {
    c_int::from(rs_ascii_isalnum(c) != 0 || c == i32::from(b'_'))
}

/// Converts ASCII lowercase letter to uppercase ('a'-'z' -> 'A'-'Z').
/// Non-lowercase letters are returned unchanged.
///
/// # Arguments
/// * `c` - The character code to convert
///
/// # Returns
/// The uppercase equivalent if 'a'-'z', otherwise the original character
#[no_mangle]
pub extern "C" fn rs_ascii_toupper(c: c_int) -> c_int {
    if c >= i32::from(b'a') && c <= i32::from(b'z') {
        c - (i32::from(b'a') - i32::from(b'A'))
    } else {
        c
    }
}

/// Converts ASCII uppercase letter to lowercase ('A'-'Z' -> 'a'-'z').
/// Non-uppercase letters are returned unchanged.
///
/// # Arguments
/// * `c` - The character code to convert
///
/// # Returns
/// The lowercase equivalent if 'A'-'Z', otherwise the original character
#[no_mangle]
pub extern "C" fn rs_ascii_tolower(c: c_int) -> c_int {
    if c >= i32::from(b'A') && c <= i32::from(b'Z') {
        c + (i32::from(b'a') - i32::from(b'A'))
    } else {
        c
    }
}

/// Returns the ordinal index of a letter (0-25).
/// If the character is lowercase ('a'-'z'), returns 0-25.
/// If the character is uppercase ('A'-'Z'), returns 0-25.
/// Otherwise returns the offset from 'A' (may be negative or >25).
///
/// # Arguments
/// * `c` - The character code
///
/// # Returns
/// The ordinal index (0-25 for valid letters)
#[no_mangle]
pub extern "C" fn rs_char_ord(c: c_int) -> c_int {
    // Original C: ((uint8_t)(x) < 'a' ? (uint8_t)(x) - 'A' : (uint8_t)(x) - 'a')
    let byte = (c & 0xFF) as u8;
    if byte < b'a' {
        i32::from(byte.wrapping_sub(b'A'))
    } else {
        i32::from(byte.wrapping_sub(b'a'))
    }
}

/// Returns the ordinal index of a lowercase letter (0-25 for 'a'-'z').
///
/// # Arguments
/// * `c` - The character code (should be 'a'-'z')
///
/// # Returns
/// The ordinal index (0-25 for 'a'-'z')
#[no_mangle]
pub extern "C" fn rs_char_ord_low(c: c_int) -> c_int {
    // Original C: ((uint8_t)(x) - 'a')
    let byte = (c & 0xFF) as u8;
    i32::from(byte.wrapping_sub(b'a'))
}

/// Returns the ordinal index of an uppercase letter (0-25 for 'A'-'Z').
///
/// # Arguments
/// * `c` - The character code (should be 'A'-'Z')
///
/// # Returns
/// The ordinal index (0-25 for 'A'-'Z')
#[no_mangle]
pub extern "C" fn rs_char_ord_up(c: c_int) -> c_int {
    // Original C: ((uint8_t)(x) - 'A')
    let byte = (c & 0xFF) as u8;
    i32::from(byte.wrapping_sub(b'A'))
}

/// ROT13 encoding - rotates a character by 13 positions within the alphabet.
///
/// # Arguments
/// * `c` - The character code to rotate
/// * `a` - The base character ('a' for lowercase, 'A' for uppercase)
///
/// # Returns
/// The ROT13 encoded character
#[no_mangle]
pub extern "C" fn rs_rot13(c: c_int, a: c_int) -> c_int {
    // Original C: (((((c) - (a)) + 13) % 26) + (a))
    (((c - a) + 13) % 26) + a
}

/// Sets the meta bit (bit 7) on a character.
///
/// # Arguments
/// * `c` - The character code
///
/// # Returns
/// The character with bit 7 set
#[no_mangle]
pub extern "C" fn rs_meta(c: c_int) -> c_int {
    // Original C: ((x) | 0x80)
    c | 0x80
}

/// Converts a character to its control character equivalent.
/// This is done by uppercasing and XORing with 0x40.
/// For example, '?' -> DEL (127), '@' -> NUL (0), 'a' or 'A' -> Ctrl-A (1)
///
/// # Arguments
/// * `c` - The character code to convert
///
/// # Returns
/// The control character equivalent
#[no_mangle]
pub extern "C" fn rs_ctrl_chr(c: c_int) -> c_int {
    // Original C: (TOUPPER_ASC(x) ^ 0x40)
    rs_ascii_toupper(c) ^ 0x40
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii_iswhite() {
        // Space and tab are whitespace
        assert_ne!(rs_ascii_iswhite(i32::from(b' ')), 0);
        assert_ne!(rs_ascii_iswhite(i32::from(b'\t')), 0);

        // Other characters are not
        assert_eq!(rs_ascii_iswhite(i32::from(b'a')), 0);
        assert_eq!(rs_ascii_iswhite(i32::from(b'\n')), 0);
        assert_eq!(rs_ascii_iswhite(0), 0);
    }

    #[test]
    fn test_ascii_iswhite_or_nul() {
        assert_ne!(rs_ascii_iswhite_or_nul(i32::from(b' ')), 0);
        assert_ne!(rs_ascii_iswhite_or_nul(i32::from(b'\t')), 0);
        assert_ne!(rs_ascii_iswhite_or_nul(0), 0);
        assert_eq!(rs_ascii_iswhite_or_nul(i32::from(b'a')), 0);
    }

    #[test]
    fn test_ascii_iswhite_nl_or_nul() {
        assert_ne!(rs_ascii_iswhite_nl_or_nul(i32::from(b' ')), 0);
        assert_ne!(rs_ascii_iswhite_nl_or_nul(i32::from(b'\t')), 0);
        assert_ne!(rs_ascii_iswhite_nl_or_nul(i32::from(b'\n')), 0);
        assert_ne!(rs_ascii_iswhite_nl_or_nul(0), 0);
        assert_eq!(rs_ascii_iswhite_nl_or_nul(i32::from(b'a')), 0);
    }

    #[test]
    fn test_ascii_isdigit() {
        for d in b'0'..=b'9' {
            assert_ne!(rs_ascii_isdigit(i32::from(d)), 0);
        }
        assert_eq!(rs_ascii_isdigit(i32::from(b'a')), 0);
        assert_eq!(rs_ascii_isdigit(i32::from(b'A')), 0);
        assert_eq!(rs_ascii_isdigit(-1), 0);
        assert_eq!(rs_ascii_isdigit(256), 0);
    }

    #[test]
    fn test_ascii_isxdigit() {
        for d in b'0'..=b'9' {
            assert_ne!(rs_ascii_isxdigit(i32::from(d)), 0);
        }
        for d in b'a'..=b'f' {
            assert_ne!(rs_ascii_isxdigit(i32::from(d)), 0);
        }
        for d in b'A'..=b'F' {
            assert_ne!(rs_ascii_isxdigit(i32::from(d)), 0);
        }
        assert_eq!(rs_ascii_isxdigit(i32::from(b'g')), 0);
        assert_eq!(rs_ascii_isxdigit(i32::from(b'G')), 0);
    }

    #[test]
    fn test_ascii_isbdigit() {
        assert_ne!(rs_ascii_isbdigit(i32::from(b'0')), 0);
        assert_ne!(rs_ascii_isbdigit(i32::from(b'1')), 0);
        assert_eq!(rs_ascii_isbdigit(i32::from(b'2')), 0);
    }

    #[test]
    fn test_ascii_isodigit() {
        for d in b'0'..=b'7' {
            assert_ne!(rs_ascii_isodigit(i32::from(d)), 0);
        }
        assert_eq!(rs_ascii_isodigit(i32::from(b'8')), 0);
        assert_eq!(rs_ascii_isodigit(i32::from(b'9')), 0);
    }

    #[test]
    fn test_ascii_isspace() {
        // All whitespace characters
        assert_ne!(rs_ascii_isspace(i32::from(b' ')), 0);
        assert_ne!(rs_ascii_isspace(i32::from(b'\t')), 0); // 9
        assert_ne!(rs_ascii_isspace(i32::from(b'\n')), 0); // 10
        assert_ne!(rs_ascii_isspace(11), 0); // \v
        assert_ne!(rs_ascii_isspace(12), 0); // \f
        assert_ne!(rs_ascii_isspace(i32::from(b'\r')), 0); // 13

        // Non-whitespace
        assert_eq!(rs_ascii_isspace(i32::from(b'a')), 0);
        assert_eq!(rs_ascii_isspace(0), 0);
    }

    #[test]
    fn test_ascii_isupper() {
        for c in b'A'..=b'Z' {
            assert_ne!(rs_ascii_isupper(i32::from(c)), 0);
        }
        assert_eq!(rs_ascii_isupper(i32::from(b'a')), 0);
        assert_eq!(rs_ascii_isupper(i32::from(b'0')), 0);
    }

    #[test]
    fn test_ascii_islower() {
        for c in b'a'..=b'z' {
            assert_ne!(rs_ascii_islower(i32::from(c)), 0);
        }
        assert_eq!(rs_ascii_islower(i32::from(b'A')), 0);
        assert_eq!(rs_ascii_islower(i32::from(b'0')), 0);
    }

    #[test]
    fn test_ascii_isalpha() {
        for c in b'A'..=b'Z' {
            assert_ne!(rs_ascii_isalpha(i32::from(c)), 0);
        }
        for c in b'a'..=b'z' {
            assert_ne!(rs_ascii_isalpha(i32::from(c)), 0);
        }
        assert_eq!(rs_ascii_isalpha(i32::from(b'0')), 0);
        assert_eq!(rs_ascii_isalpha(i32::from(b'_')), 0);
    }

    #[test]
    fn test_ascii_isalnum() {
        for c in b'A'..=b'Z' {
            assert_ne!(rs_ascii_isalnum(i32::from(c)), 0);
        }
        for c in b'a'..=b'z' {
            assert_ne!(rs_ascii_isalnum(i32::from(c)), 0);
        }
        for c in b'0'..=b'9' {
            assert_ne!(rs_ascii_isalnum(i32::from(c)), 0);
        }
        assert_eq!(rs_ascii_isalnum(i32::from(b'_')), 0);
        assert_eq!(rs_ascii_isalnum(i32::from(b' ')), 0);
    }

    #[test]
    fn test_ascii_isident() {
        for c in b'A'..=b'Z' {
            assert_ne!(rs_ascii_isident(i32::from(c)), 0);
        }
        for c in b'a'..=b'z' {
            assert_ne!(rs_ascii_isident(i32::from(c)), 0);
        }
        for c in b'0'..=b'9' {
            assert_ne!(rs_ascii_isident(i32::from(c)), 0);
        }
        assert_ne!(rs_ascii_isident(i32::from(b'_')), 0);
        assert_eq!(rs_ascii_isident(i32::from(b' ')), 0);
        assert_eq!(rs_ascii_isident(i32::from(b'-')), 0);
    }

    #[test]
    fn test_ascii_toupper() {
        // Lowercase letters are converted to uppercase
        for c in b'a'..=b'z' {
            let expected = c - (b'a' - b'A');
            assert_eq!(rs_ascii_toupper(i32::from(c)), i32::from(expected));
        }
        // Uppercase letters are unchanged
        for c in b'A'..=b'Z' {
            assert_eq!(rs_ascii_toupper(i32::from(c)), i32::from(c));
        }
        // Other characters are unchanged
        assert_eq!(rs_ascii_toupper(i32::from(b'0')), i32::from(b'0'));
        assert_eq!(rs_ascii_toupper(i32::from(b' ')), i32::from(b' '));
        assert_eq!(rs_ascii_toupper(i32::from(b'_')), i32::from(b'_'));
        assert_eq!(rs_ascii_toupper(-1), -1);
        assert_eq!(rs_ascii_toupper(256), 256);
    }

    #[test]
    fn test_ascii_tolower() {
        // Uppercase letters are converted to lowercase
        for c in b'A'..=b'Z' {
            let expected = c + (b'a' - b'A');
            assert_eq!(rs_ascii_tolower(i32::from(c)), i32::from(expected));
        }
        // Lowercase letters are unchanged
        for c in b'a'..=b'z' {
            assert_eq!(rs_ascii_tolower(i32::from(c)), i32::from(c));
        }
        // Other characters are unchanged
        assert_eq!(rs_ascii_tolower(i32::from(b'0')), i32::from(b'0'));
        assert_eq!(rs_ascii_tolower(i32::from(b' ')), i32::from(b' '));
        assert_eq!(rs_ascii_tolower(i32::from(b'_')), i32::from(b'_'));
        assert_eq!(rs_ascii_tolower(-1), -1);
        assert_eq!(rs_ascii_tolower(256), 256);
    }

    #[test]
    fn test_char_ord() {
        // Lowercase letters return 0-25
        for (i, c) in (b'a'..=b'z').enumerate() {
            assert_eq!(rs_char_ord(i32::from(c)), i as i32);
        }
        // Uppercase letters also return 0-25
        for (i, c) in (b'A'..=b'Z').enumerate() {
            assert_eq!(rs_char_ord(i32::from(c)), i as i32);
        }
    }

    #[test]
    fn test_char_ord_low() {
        // Lowercase letters return 0-25
        for (i, c) in (b'a'..=b'z').enumerate() {
            assert_eq!(rs_char_ord_low(i32::from(c)), i as i32);
        }
    }

    #[test]
    fn test_char_ord_up() {
        // Uppercase letters return 0-25
        for (i, c) in (b'A'..=b'Z').enumerate() {
            assert_eq!(rs_char_ord_up(i32::from(c)), i as i32);
        }
    }

    #[test]
    fn test_rot13() {
        // ROT13 of 'a' with base 'a' should be 'n'
        assert_eq!(rs_rot13(i32::from(b'a'), i32::from(b'a')), i32::from(b'n'));
        // ROT13 of 'n' with base 'a' should be 'a' (ROT13 is self-inverse)
        assert_eq!(rs_rot13(i32::from(b'n'), i32::from(b'a')), i32::from(b'a'));
        // ROT13 of 'z' with base 'a' should be 'm'
        assert_eq!(rs_rot13(i32::from(b'z'), i32::from(b'a')), i32::from(b'm'));
        // ROT13 of 'A' with base 'A' should be 'N'
        assert_eq!(rs_rot13(i32::from(b'A'), i32::from(b'A')), i32::from(b'N'));
    }

    #[test]
    fn test_meta() {
        // Setting meta bit on 'a' (0x61) gives 0xE1
        assert_eq!(rs_meta(i32::from(b'a')), 0xE1);
        // Setting meta bit on 0 gives 0x80
        assert_eq!(rs_meta(0), 0x80);
        // Setting meta bit on something that already has it is idempotent
        assert_eq!(rs_meta(0x80), 0x80);
    }

    #[test]
    fn test_ctrl_chr() {
        // '?' -> DEL (127)
        assert_eq!(rs_ctrl_chr(i32::from(b'?')), 127);
        // '@' -> NUL (0)
        assert_eq!(rs_ctrl_chr(i32::from(b'@')), 0);
        // 'A' -> Ctrl-A (1)
        assert_eq!(rs_ctrl_chr(i32::from(b'A')), 1);
        // 'a' -> Ctrl-A (1) (case-insensitive)
        assert_eq!(rs_ctrl_chr(i32::from(b'a')), 1);
        // 'Z' -> Ctrl-Z (26)
        assert_eq!(rs_ctrl_chr(i32::from(b'Z')), 26);
    }
}
