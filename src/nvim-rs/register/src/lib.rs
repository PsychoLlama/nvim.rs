//! Register utilities for Neovim
//!
//! This crate provides functions for validating register names and operations.

use std::ffi::c_int;

/// Register index constants (matching `register_defs.h`).
pub const DELETION_REGISTER: c_int = 36;
pub const STAR_REGISTER: c_int = 37;
pub const PLUS_REGISTER: c_int = 38;

/// Check if a character is an ASCII alphanumeric character (A-Z, a-z, 0-9).
#[inline]
const fn ascii_isalnum(c: u8) -> bool {
    (c >= b'A' && c <= b'Z') || (c >= b'a' && c <= b'z') || (c >= b'0' && c <= b'9')
}

/// Check if a character is an ASCII digit (0-9).
#[inline]
const fn ascii_isdigit(c: u8) -> bool {
    c >= b'0' && c <= b'9'
}

/// Check if a character is an ASCII lowercase letter (a-z).
#[inline]
const fn ascii_islower(c: u8) -> bool {
    c >= b'a' && c <= b'z'
}

/// Check if a character is an ASCII uppercase letter (A-Z).
#[inline]
const fn ascii_isupper(c: u8) -> bool {
    c >= b'A' && c <= b'Z'
}

/// Check if a character appears in a string.
#[inline]
fn strchr(s: &[u8], c: u8) -> bool {
    s.contains(&c)
}

/// Check if register should be inserted literally (selection or clipboard).
///
/// Returns true for '*', '+', or any alphanumeric register name.
#[no_mangle]
pub extern "C" fn rs_is_literal_register(regname: c_int) -> c_int {
    let Ok(c) = u8::try_from(regname) else {
        return 0;
    };
    c_int::from(c == b'*' || c == b'+' || ascii_isalnum(c))
}

/// Convert register name into register index.
///
/// Returns the index in the `y_regs` array, or -1 if the register name is not recognized.
#[no_mangle]
pub extern "C" fn rs_op_reg_index(regname: c_int) -> c_int {
    let Ok(c) = u8::try_from(regname) else {
        return -1;
    };
    if ascii_isdigit(c) {
        // Digits 0-9 map to indices 0-9
        c_int::from(c - b'0')
    } else if ascii_islower(c) {
        // Lowercase a-z maps to indices 10-35
        c_int::from(c - b'a') + 10
    } else if ascii_isupper(c) {
        // Uppercase A-Z maps to indices 10-35 (same as lowercase)
        c_int::from(c - b'A') + 10
    } else if c == b'-' {
        DELETION_REGISTER
    } else if c == b'*' {
        STAR_REGISTER
    } else if c == b'+' {
        PLUS_REGISTER
    } else {
        -1
    }
}

/// Check if register name indicates append mode (uppercase letter).
#[no_mangle]
pub extern "C" fn rs_is_append_register(regname: c_int) -> c_int {
    let Ok(c) = u8::try_from(regname) else {
        return 0;
    };
    c_int::from(ascii_isupper(c))
}

/// Get the character name of the register with the given index.
///
/// Returns the register character name, or '"' for index -1.
#[no_mangle]
pub extern "C" fn rs_get_register_name(num: c_int) -> c_int {
    if num == -1 {
        c_int::from(b'"')
    } else if num < 10 {
        num + c_int::from(b'0')
    } else if num == DELETION_REGISTER {
        c_int::from(b'-')
    } else if num == STAR_REGISTER {
        c_int::from(b'*')
    } else if num == PLUS_REGISTER {
        c_int::from(b'+')
    } else {
        num + c_int::from(b'a') - 10
    }
}

/// Check if `regname` is a valid name of a yank register.
///
/// There is no check for 0 (default register), caller should do this.
/// The black hole register '_' is regarded as valid.
///
/// # Arguments
///
/// * `regname` - name of register (as a character code)
/// * `writing` - allow only writable registers
///
/// # Returns
///
/// `true` if the register name is valid
#[no_mangle]
pub extern "C" fn rs_valid_yank_reg(regname: c_int, writing: bool) -> bool {
    // Convert to u8, invalid values return false
    let Ok(c) = u8::try_from(regname) else {
        return false;
    };

    // Named registers (a-z, A-Z, 0-9)
    if regname > 0 && ascii_isalnum(c) {
        return true;
    }

    // Read-only registers (only valid when not writing): . / % : =
    if !writing && strchr(b"/.%:=", c) {
        return true;
    }

    // Special registers: # " - _ * +
    matches!(c, b'#' | b'"' | b'-' | b'_' | b'*' | b'+')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_yank_reg_named() {
        // Alphabetic registers (a-z, A-Z)
        assert!(rs_valid_yank_reg(c_int::from(b'a'), false));
        assert!(rs_valid_yank_reg(c_int::from(b'z'), false));
        assert!(rs_valid_yank_reg(c_int::from(b'A'), false));
        assert!(rs_valid_yank_reg(c_int::from(b'Z'), false));
        assert!(rs_valid_yank_reg(c_int::from(b'a'), true));

        // Numeric registers (0-9)
        assert!(rs_valid_yank_reg(c_int::from(b'0'), false));
        assert!(rs_valid_yank_reg(c_int::from(b'9'), false));
        assert!(rs_valid_yank_reg(c_int::from(b'5'), true));
    }

    #[test]
    fn test_valid_yank_reg_readonly() {
        // Read-only registers: . / % : =
        // These are only valid when NOT writing
        assert!(rs_valid_yank_reg(c_int::from(b'.'), false));
        assert!(rs_valid_yank_reg(c_int::from(b'/'), false));
        assert!(rs_valid_yank_reg(c_int::from(b'%'), false));
        assert!(rs_valid_yank_reg(c_int::from(b':'), false));
        assert!(rs_valid_yank_reg(c_int::from(b'='), false));

        // Not valid when writing
        assert!(!rs_valid_yank_reg(c_int::from(b'.'), true));
        assert!(!rs_valid_yank_reg(c_int::from(b'/'), true));
        assert!(!rs_valid_yank_reg(c_int::from(b'%'), true));
        assert!(!rs_valid_yank_reg(c_int::from(b':'), true));
        assert!(!rs_valid_yank_reg(c_int::from(b'='), true));
    }

    #[test]
    fn test_valid_yank_reg_special() {
        // Special registers: # " - _ * +
        assert!(rs_valid_yank_reg(c_int::from(b'#'), false));
        assert!(rs_valid_yank_reg(c_int::from(b'"'), false));
        assert!(rs_valid_yank_reg(c_int::from(b'-'), false));
        assert!(rs_valid_yank_reg(c_int::from(b'_'), false));
        assert!(rs_valid_yank_reg(c_int::from(b'*'), false));
        assert!(rs_valid_yank_reg(c_int::from(b'+'), false));

        // Also valid when writing
        assert!(rs_valid_yank_reg(c_int::from(b'#'), true));
        assert!(rs_valid_yank_reg(c_int::from(b'"'), true));
        assert!(rs_valid_yank_reg(c_int::from(b'-'), true));
        assert!(rs_valid_yank_reg(c_int::from(b'_'), true));
        assert!(rs_valid_yank_reg(c_int::from(b'*'), true));
        assert!(rs_valid_yank_reg(c_int::from(b'+'), true));
    }

    #[test]
    fn test_valid_yank_reg_invalid() {
        // Invalid register names
        assert!(!rs_valid_yank_reg(c_int::from(b' '), false));
        assert!(!rs_valid_yank_reg(c_int::from(b'!'), false));
        assert!(!rs_valid_yank_reg(c_int::from(b'@'), false));
        assert!(!rs_valid_yank_reg(c_int::from(b'$'), false));
        assert!(!rs_valid_yank_reg(c_int::from(b'^'), false));
        assert!(!rs_valid_yank_reg(c_int::from(b'&'), false));
        assert!(!rs_valid_yank_reg(c_int::from(b'('), false));
        assert!(!rs_valid_yank_reg(c_int::from(b')'), false));

        // Negative values
        assert!(!rs_valid_yank_reg(-1, false));

        // Values > 255
        assert!(!rs_valid_yank_reg(256, false));

        // Zero (not handled by this function - caller's responsibility)
        assert!(!rs_valid_yank_reg(0, false));
    }

    #[test]
    fn test_is_literal_register() {
        // Alphanumeric registers are literal
        assert_ne!(rs_is_literal_register(c_int::from(b'a')), 0);
        assert_ne!(rs_is_literal_register(c_int::from(b'Z')), 0);
        assert_ne!(rs_is_literal_register(c_int::from(b'0')), 0);

        // Star and plus are literal
        assert_ne!(rs_is_literal_register(c_int::from(b'*')), 0);
        assert_ne!(rs_is_literal_register(c_int::from(b'+')), 0);

        // Other special registers are not literal
        assert_eq!(rs_is_literal_register(c_int::from(b'-')), 0);
        assert_eq!(rs_is_literal_register(c_int::from(b'"')), 0);
        assert_eq!(rs_is_literal_register(c_int::from(b'#')), 0);
    }

    #[test]
    fn test_op_reg_index() {
        // Digits map to 0-9
        assert_eq!(rs_op_reg_index(c_int::from(b'0')), 0);
        assert_eq!(rs_op_reg_index(c_int::from(b'9')), 9);

        // Lowercase letters map to 10-35
        assert_eq!(rs_op_reg_index(c_int::from(b'a')), 10);
        assert_eq!(rs_op_reg_index(c_int::from(b'z')), 35);

        // Uppercase letters also map to 10-35
        assert_eq!(rs_op_reg_index(c_int::from(b'A')), 10);
        assert_eq!(rs_op_reg_index(c_int::from(b'Z')), 35);

        // Special registers
        assert_eq!(rs_op_reg_index(c_int::from(b'-')), DELETION_REGISTER);
        assert_eq!(rs_op_reg_index(c_int::from(b'*')), STAR_REGISTER);
        assert_eq!(rs_op_reg_index(c_int::from(b'+')), PLUS_REGISTER);

        // Invalid returns -1
        assert_eq!(rs_op_reg_index(c_int::from(b'@')), -1);
        assert_eq!(rs_op_reg_index(-1), -1);
        assert_eq!(rs_op_reg_index(256), -1);
    }

    #[test]
    fn test_is_append_register() {
        // Uppercase letters are append registers
        assert_ne!(rs_is_append_register(c_int::from(b'A')), 0);
        assert_ne!(rs_is_append_register(c_int::from(b'Z')), 0);

        // Lowercase letters are not append registers
        assert_eq!(rs_is_append_register(c_int::from(b'a')), 0);
        assert_eq!(rs_is_append_register(c_int::from(b'z')), 0);

        // Other characters are not append registers
        assert_eq!(rs_is_append_register(c_int::from(b'0')), 0);
        assert_eq!(rs_is_append_register(c_int::from(b'-')), 0);
    }

    #[test]
    fn test_get_register_name() {
        // -1 returns '"'
        assert_eq!(rs_get_register_name(-1), c_int::from(b'"'));

        // 0-9 return '0'-'9'
        assert_eq!(rs_get_register_name(0), c_int::from(b'0'));
        assert_eq!(rs_get_register_name(9), c_int::from(b'9'));

        // 10-35 return 'a'-'z'
        assert_eq!(rs_get_register_name(10), c_int::from(b'a'));
        assert_eq!(rs_get_register_name(35), c_int::from(b'z'));

        // Special registers
        assert_eq!(rs_get_register_name(DELETION_REGISTER), c_int::from(b'-'));
        assert_eq!(rs_get_register_name(STAR_REGISTER), c_int::from(b'*'));
        assert_eq!(rs_get_register_name(PLUS_REGISTER), c_int::from(b'+'));
    }
}
