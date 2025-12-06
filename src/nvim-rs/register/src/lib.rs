//! Register utilities for Neovim
//!
//! This crate provides functions for validating register names and operations.

use std::ffi::c_int;

/// Check if a character is an ASCII alphanumeric character (A-Z, a-z, 0-9).
#[inline]
const fn ascii_isalnum(c: u8) -> bool {
    (c >= b'A' && c <= b'Z') || (c >= b'a' && c <= b'z') || (c >= b'0' && c <= b'9')
}

/// Check if a character appears in a string.
#[inline]
fn strchr(s: &[u8], c: u8) -> bool {
    s.contains(&c)
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
}
