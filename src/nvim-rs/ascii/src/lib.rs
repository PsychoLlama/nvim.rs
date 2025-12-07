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
}
