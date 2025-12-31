//! Register utilities for Neovim
//!
//! This crate provides functions for validating register names and operations.

#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::must_use_candidate)]

use std::ffi::{c_char, c_int};

/// MotionType values matching `normal_defs.h`.
pub const K_MT_CHAR_WISE: c_int = 0;
pub const K_MT_LINE_WISE: c_int = 1;
pub const K_MT_BLOCK_WISE: c_int = 2;
pub const K_MT_UNKNOWN: c_int = -1;

/// CTRL-V character (0x16 = 22 decimal).
const CTRL_V: u8 = 0x16;

/// Opaque handle to yankreg_T.
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct YankRegHandle(*mut std::ffi::c_void);

// FFI declarations for C accessor functions
extern "C" {
    /// Get the index of the unnamed register (y_previous - y_regs), or -1 if NULL.
    fn nvim_get_y_previous_index() -> c_int;

    // yankreg_T accessors
    fn nvim_yankreg_get_size(reg: YankRegHandle) -> usize;
    fn nvim_yankreg_get_type(reg: YankRegHandle) -> c_int;
    fn nvim_yankreg_get_width(reg: YankRegHandle) -> c_int;
    fn nvim_yankreg_set_width(reg: YankRegHandle, width: c_int);
    fn nvim_yankreg_get_line_data(reg: YankRegHandle, idx: usize) -> *const c_char;
    fn nvim_yankreg_get_line_size(reg: YankRegHandle, idx: usize) -> usize;
    fn nvim_yankreg_is_empty(reg: YankRegHandle) -> bool;

    // Global register array accessors
    fn nvim_get_y_regs_ptr(idx: c_int) -> YankRegHandle;
    fn nvim_set_y_previous_by_index(idx: c_int);
    fn nvim_free_register(reg: YankRegHandle);
    fn nvim_copy_yankreg(dst: YankRegHandle, src: YankRegHandle);
    fn nvim_clear_yankreg_array(reg: YankRegHandle);

    // mbyte function for calculating string width
    fn rs_mb_string2cells_len(str: *const c_char, size: usize) -> usize;
}

/// Register index constants (matching `register_defs.h`).
pub const DELETION_REGISTER: c_int = 36;
pub const NUM_SAVED_REGISTERS: c_int = 37;
pub const STAR_REGISTER: c_int = 37;
pub const PLUS_REGISTER: c_int = 38;
pub const NUM_REGISTERS: c_int = 39;

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

/// Get the index of the register that "" points to.
///
/// Returns the index of `y_previous` in the `y_regs` array, or -1 if
/// `y_previous` is NULL (no previous yank).
///
/// # Safety
///
/// Calls external C function to access global register state.
#[no_mangle]
pub unsafe extern "C" fn rs_get_unname_register() -> c_int {
    nvim_get_y_previous_index()
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

/// Updates the "y_width" of a blockwise register based on its contents.
/// Does nothing on a non-blockwise register.
///
/// # Safety
///
/// The `reg` handle must be a valid pointer to a yankreg_T.
#[no_mangle]
pub unsafe extern "C" fn rs_update_yankreg_width(reg: YankRegHandle) {
    if reg.0.is_null() {
        return;
    }

    let reg_type = nvim_yankreg_get_type(reg);
    if reg_type != K_MT_BLOCK_WISE {
        return;
    }

    let y_size = nvim_yankreg_get_size(reg);
    let mut maxlen: usize = 0;

    for i in 0..y_size {
        let data = nvim_yankreg_get_line_data(reg, i);
        let size = nvim_yankreg_get_line_size(reg, i);
        let rowlen = rs_mb_string2cells_len(data, size);
        maxlen = maxlen.max(rowlen);
    }

    let current_width = nvim_yankreg_get_width(reg);
    // maxlen - 1, but maxlen can be 0
    let new_width = if maxlen > 0 { (maxlen - 1) as c_int } else { 0 };
    nvim_yankreg_set_width(reg, current_width.max(new_width));
}

/// Get the number of non-empty registers.
///
/// # Safety
///
/// Accesses global register state via C FFI.
#[no_mangle]
pub unsafe extern "C" fn rs_op_reg_amount() -> usize {
    let mut count: usize = 0;
    for i in 0..NUM_SAVED_REGISTERS {
        let reg = nvim_get_y_regs_ptr(i);
        if !nvim_yankreg_is_empty(reg) {
            count += 1;
        }
    }
    count
}

/// Get register with the given name.
///
/// Returns a pointer to the register contents, or NULL if the register name is invalid.
///
/// # Safety
///
/// Accesses global register state via C FFI.
#[no_mangle]
pub unsafe extern "C" fn rs_op_reg_get(name: c_char) -> YankRegHandle {
    let i = rs_op_reg_index(c_int::from(name));
    if i == -1 {
        return YankRegHandle(std::ptr::null_mut());
    }
    nvim_get_y_regs_ptr(i)
}

/// Set the previous yank register.
///
/// Returns true on success, false if the register name is invalid.
///
/// # Safety
///
/// Modifies global register state via C FFI.
#[no_mangle]
pub unsafe extern "C" fn rs_op_reg_set_previous(name: c_char) -> bool {
    let i = rs_op_reg_index(c_int::from(name));
    if i == -1 {
        return false;
    }
    nvim_set_y_previous_by_index(i);
    true
}

/// Shift the delete registers: "9 is cleared, "8 becomes "9, etc.
///
/// # Safety
///
/// Modifies global register state via C FFI.
#[no_mangle]
pub unsafe extern "C" fn rs_shift_delete_registers(y_append: bool) {
    // Free register "9
    let reg9 = nvim_get_y_regs_ptr(9);
    nvim_free_register(reg9);

    // Shift registers: 9 <- 8 <- 7 <- ... <- 2 <- 1
    for n in (2..=9).rev() {
        let dst = nvim_get_y_regs_ptr(n);
        let src = nvim_get_y_regs_ptr(n - 1);
        nvim_copy_yankreg(dst, src);
    }

    // Set y_previous to register "1 if not appending
    if !y_append {
        nvim_set_y_previous_by_index(1);
    }

    // Set register "1 to empty
    let reg1 = nvim_get_y_regs_ptr(1);
    nvim_clear_yankreg_array(reg1);
}

/// Format the register type as a string.
///
/// # Arguments
///
/// * `reg_type` - The register type (MotionType).
/// * `reg_width` - The width, only used if "reg_type" is kMTBlockWise.
/// * `buf` - Buffer to store formatted string.
/// * `buf_len` - The allocated size of the buffer.
///
/// # Safety
///
/// The `buf` pointer must be valid and point to a buffer of at least `buf_len` bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_format_reg_type(
    reg_type: c_int,
    reg_width: c_int,
    buf: *mut c_char,
    buf_len: usize,
) {
    if buf.is_null() || buf_len < 2 {
        return;
    }

    let buf_slice = std::slice::from_raw_parts_mut(buf as *mut u8, buf_len);

    match reg_type {
        K_MT_LINE_WISE => {
            buf_slice[0] = b'V';
            buf_slice[1] = 0;
        }
        K_MT_CHAR_WISE => {
            buf_slice[0] = b'v';
            buf_slice[1] = 0;
        }
        K_MT_BLOCK_WISE => {
            // Format as ^V{width+1}
            let width = reg_width + 1;
            let formatted = format!("{}", width);
            let formatted_bytes = formatted.as_bytes();

            buf_slice[0] = CTRL_V;
            let copy_len = formatted_bytes.len().min(buf_len - 2);
            buf_slice[1..1 + copy_len].copy_from_slice(&formatted_bytes[..copy_len]);
            buf_slice[1 + copy_len] = 0;
        }
        _ => {
            // kMTUnknown or invalid
            buf_slice[0] = 0;
        }
    }
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

    #[test]
    fn test_format_reg_type() {
        let mut buf = [0u8; 20];

        // kMTLineWise -> 'V'
        unsafe {
            rs_format_reg_type(
                K_MT_LINE_WISE,
                0,
                buf.as_mut_ptr() as *mut c_char,
                buf.len(),
            );
        }
        assert_eq!(buf[0], b'V');
        assert_eq!(buf[1], 0);

        // kMTCharWise -> 'v'
        buf = [0u8; 20];
        unsafe {
            rs_format_reg_type(
                K_MT_CHAR_WISE,
                0,
                buf.as_mut_ptr() as *mut c_char,
                buf.len(),
            );
        }
        assert_eq!(buf[0], b'v');
        assert_eq!(buf[1], 0);

        // kMTBlockWise -> ^V{width+1}
        buf = [0u8; 20];
        unsafe {
            rs_format_reg_type(
                K_MT_BLOCK_WISE,
                9,
                buf.as_mut_ptr() as *mut c_char,
                buf.len(),
            );
        }
        assert_eq!(buf[0], CTRL_V);
        assert_eq!(buf[1], b'1');
        assert_eq!(buf[2], b'0');
        assert_eq!(buf[3], 0);

        // kMTBlockWise with width 0 -> ^V1
        buf = [0u8; 20];
        unsafe {
            rs_format_reg_type(
                K_MT_BLOCK_WISE,
                0,
                buf.as_mut_ptr() as *mut c_char,
                buf.len(),
            );
        }
        assert_eq!(buf[0], CTRL_V);
        assert_eq!(buf[1], b'1');
        assert_eq!(buf[2], 0);

        // kMTUnknown -> empty
        buf = [0u8; 20];
        unsafe {
            rs_format_reg_type(K_MT_UNKNOWN, 0, buf.as_mut_ptr() as *mut c_char, buf.len());
        }
        assert_eq!(buf[0], 0);
    }
}
