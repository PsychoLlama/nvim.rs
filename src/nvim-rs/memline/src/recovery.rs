//! Recovery and attention file handling for the memline system.
//!
//! This module provides Rust wrappers for swap file recovery operations,
//! including:
//! - Finding and listing swap files for recovery
//! - Reading swap file metadata (block 0 info)
//! - Handling the "ATTENTION - swap file exists" dialog
//! - Swap file name resolution
//!
//! # Recovery Process
//!
//! When Neovim encounters an existing swap file, it can:
//! 1. List available swap files with `recover_names()`
//! 2. Show swap file info with `swapfile_info()` / `swapfile_dict()`
//! 3. Present the ATTENTION dialog with `attention_message()`
//! 4. Recover content with `ml_recover()`
//!
//! # Safety
//!
//! Recovery operations read from potentially corrupted files and should
//! be prepared to handle invalid data gracefully.

use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// C Implementation Declarations
// =============================================================================

extern "C" {
    // -------------------------------------------------------------------------
    // Recovery Operations (C implementations)
    // -------------------------------------------------------------------------

    /// Recover from a swap file
    fn ml_recover(checkext: c_int);

    /// Get list of swap files for recovery
    fn recover_names(
        fname: *const c_char,
        do_list: c_int,
        ret_list: *mut c_void,
        nr: c_int,
        fname_out: *mut *mut c_char,
    ) -> c_int;

    /// Write swap file info to a dictionary
    fn swapfile_dict(fname: *const c_char, d: *mut c_void);

    /// Resolve symlinks in a path
    fn resolve_symlink(fname: *const c_char, buf: *mut c_char) -> c_int;

    /// Make a percent-encoded swap file name
    fn make_percent_swname(
        dir: *mut c_char,
        dir_end: *mut c_char,
        name: *const c_char,
    ) -> *mut c_char;
}

// =============================================================================
// Recovery Functions
// =============================================================================

/// Recover the contents of a buffer from its swap file.
///
/// This reads the swap file and reconstructs the buffer content.
/// If `checkext` is true, verify the file extension is correct.
///
/// # Safety
/// Modifies buffer state and may read from corrupted files.
#[no_mangle]
pub unsafe extern "C" fn rs_ml_recover(checkext: c_int) {
    ml_recover(checkext);
}

/// Get the number of swap files for a given file name.
///
/// # Arguments
/// * `fname` - File name to search for (or NULL to search all)
///
/// # Returns
/// Number of swap files found
///
/// # Safety
/// - `fname` must be a valid C string or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_recover_names_count(fname: *const c_char) -> c_int {
    recover_names(fname, 0, std::ptr::null_mut(), 0, std::ptr::null_mut())
}

/// List all swap files for a given file name.
///
/// Displays the list to the user via messaging.
///
/// # Arguments
/// * `fname` - File name to search for (or NULL to search all)
///
/// # Safety
/// - `fname` must be a valid C string or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_recover_names_list(fname: *const c_char) {
    recover_names(fname, 1, std::ptr::null_mut(), 0, std::ptr::null_mut());
}

/// Get the Nth swap file name for a given file.
///
/// # Arguments
/// * `fname` - File name to search for
/// * `nr` - Which swap file to return (1-based)
/// * `fname_out` - Output: the swap file name (caller must free)
///
/// # Returns
/// Number of swap files found
///
/// # Safety
/// - `fname` must be a valid C string or NULL
/// - `fname_out` must be a valid pointer
/// - The returned string must be freed by the caller
#[no_mangle]
pub unsafe extern "C" fn rs_recover_names_get(
    fname: *const c_char,
    nr: c_int,
    fname_out: *mut *mut c_char,
) -> c_int {
    if fname_out.is_null() {
        return 0;
    }
    recover_names(fname, 0, std::ptr::null_mut(), nr, fname_out)
}

/// Get the list of swap files as a Vim list.
///
/// # Arguments
/// * `fname` - File name to search for (or NULL to search all)
/// * `ret_list` - Vim list_T to populate
///
/// # Returns
/// Number of swap files found
///
/// # Safety
/// - `fname` must be a valid C string or NULL
/// - `ret_list` must be a valid list_T pointer
#[no_mangle]
pub unsafe extern "C" fn rs_recover_names_to_list(
    fname: *const c_char,
    ret_list: *mut c_void,
) -> c_int {
    if ret_list.is_null() {
        return 0;
    }
    recover_names(fname, 0, ret_list, 0, std::ptr::null_mut())
}

// =============================================================================
// Swap File Information
// =============================================================================

/// Write swap file information to a dictionary.
///
/// Reads the block 0 header from the swap file and populates the
/// dictionary with version, user, host, filename, pid, mtime, etc.
///
/// # Arguments
/// * `fname` - Path to the swap file
/// * `d` - Dictionary to populate
///
/// # Safety
/// - `fname` must be a valid C string or NULL
/// - `d` must be a valid dict_T pointer or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_swapfile_dict(fname: *const c_char, d: *mut c_void) {
    if fname.is_null() || d.is_null() {
        return;
    }
    swapfile_dict(fname, d);
}

// =============================================================================
// Path Utilities
// =============================================================================

/// Resolve symlinks in a file path.
///
/// If the file is a symlink, resolves it to the actual file.
/// This is important because swap files use the real path.
///
/// # Arguments
/// * `fname` - Input file path
/// * `buf` - Output buffer for resolved path (must be MAXPATHL bytes)
///
/// # Returns
/// OK if resolved, FAIL otherwise
///
/// # Safety
/// - `fname` must be a valid C string
/// - `buf` must be a valid buffer of at least MAXPATHL bytes
#[no_mangle]
pub unsafe extern "C" fn rs_resolve_symlink(fname: *const c_char, buf: *mut c_char) -> c_int {
    if fname.is_null() || buf.is_null() {
        return 1; // FAIL
    }
    resolve_symlink(fname, buf)
}

/// Make a percent-encoded swap file name for a full path.
///
/// When swap files are stored in a single directory with `//` at the end,
/// the full path is encoded with `%` characters to avoid conflicts.
///
/// # Arguments
/// * `dir` - Directory path
/// * `dir_end` - Pointer to end of significant part of dir
/// * `name` - File name to encode
///
/// # Returns
/// Allocated swap file name, or NULL on error
///
/// # Safety
/// - All pointers must be valid C strings
/// - The returned pointer must be freed by the caller
#[no_mangle]
pub unsafe extern "C" fn rs_make_percent_swname(
    dir: *mut c_char,
    dir_end: *mut c_char,
    name: *const c_char,
) -> *mut c_char {
    if dir.is_null() || dir_end.is_null() || name.is_null() {
        return std::ptr::null_mut();
    }
    make_percent_swname(dir, dir_end, name)
}

// =============================================================================
// Block 0 Validation
// =============================================================================

extern "C" {
    /// Check if block 0 has valid ID bytes
    fn ml_check_b0_id_c(b0: *const c_void) -> c_int;

    /// Check if block 0 has valid strings
    fn ml_check_b0_strings_c(b0: *const c_void) -> c_int;

    /// Check if block 0 has wrong magic number (endianness)
    fn b0_magic_wrong_c(b0: *const c_void) -> c_int;
}

/// Check if a block 0 has valid identification bytes.
///
/// Block 0 must start with BLOCK0_ID0 and BLOCK0_ID1.
///
/// # Safety
/// - `b0` must be a valid pointer to a ZeroBlock
#[no_mangle]
pub unsafe extern "C" fn rs_ml_check_b0_id(b0: *const c_void) -> c_int {
    if b0.is_null() {
        return 1; // FAIL
    }
    ml_check_b0_id_c(b0)
}

/// Check if block 0 strings are valid (NUL-terminated).
///
/// # Safety
/// - `b0` must be a valid pointer to a ZeroBlock
#[no_mangle]
pub unsafe extern "C" fn rs_ml_check_b0_strings(b0: *const c_void) -> c_int {
    if b0.is_null() {
        return 1; // FAIL
    }
    ml_check_b0_strings_c(b0)
}

/// Check if block 0 has wrong byte order (magic number check).
///
/// Returns true if the magic numbers don't match expected values,
/// indicating the swap file was created on a system with different
/// byte order.
///
/// # Safety
/// - `b0` must be a valid pointer to a ZeroBlock
#[no_mangle]
pub unsafe extern "C" fn rs_b0_magic_wrong(b0: *const c_void) -> c_int {
    if b0.is_null() {
        return 1; // true, it's wrong
    }
    b0_magic_wrong_c(b0)
}

// =============================================================================
// Byte Order Utilities
// =============================================================================

extern "C" {
    /// Convert long to char array (for swap file)
    fn long_to_char_c(n: i64, s: *mut c_char);

    /// Convert char array to long (from swap file)
    fn char_to_long_c(s: *const c_char) -> i64;
}

/// Convert a long integer to a byte array for swap file storage.
///
/// The bytes are stored in a portable format that can be read
/// regardless of the machine's byte order.
///
/// # Arguments
/// * `n` - The number to convert
/// * `s` - Output buffer (must be at least 8 bytes)
///
/// # Safety
/// - `s` must be a valid buffer of at least 8 bytes
#[no_mangle]
pub unsafe extern "C" fn rs_long_to_char(n: i64, s: *mut c_char) {
    if s.is_null() {
        return;
    }
    long_to_char_c(n, s);
}

/// Convert a byte array from swap file storage to a long integer.
///
/// Reverses the `long_to_char()` operation.
///
/// # Arguments
/// * `s` - Input buffer (must be at least 8 bytes)
///
/// # Returns
/// The decoded integer value
///
/// # Safety
/// - `s` must be a valid buffer of at least 8 bytes
#[no_mangle]
pub unsafe extern "C" fn rs_char_to_long(s: *const c_char) -> i64 {
    if s.is_null() {
        return 0;
    }
    char_to_long_c(s)
}

// =============================================================================
// File Name Comparison
// =============================================================================

extern "C" {
    /// Compare file names considering inode
    fn fnamecmp_ino_c(fname_c: *const c_char, fname_s: *const c_char, ino_block0: i64) -> c_int;
}

/// Compare two file names, considering inode number.
///
/// Used during recovery to match the current file with the file
/// recorded in the swap file's block 0.
///
/// # Arguments
/// * `fname_c` - Current file name
/// * `fname_s` - File name from swap file
/// * `ino_block0` - Inode from block 0
///
/// # Returns
/// true if files match, false otherwise
///
/// # Safety
/// - Both file name pointers must be valid C strings or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_fnamecmp_ino(
    fname_c: *const c_char,
    fname_s: *const c_char,
    ino_block0: i64,
) -> c_int {
    if fname_c.is_null() || fname_s.is_null() {
        return 0; // false, don't match
    }
    fnamecmp_ino_c(fname_c, fname_s, ino_block0)
}

// =============================================================================
// Recovery State Helpers
// =============================================================================

use crate::types::{
    SEA_CHOICE_ABORT, SEA_CHOICE_DELETE, SEA_CHOICE_EDIT, SEA_CHOICE_NONE, SEA_CHOICE_QUIT,
    SEA_CHOICE_READONLY, SEA_CHOICE_RECOVER,
};

/// Check if a swap file can be recovered.
///
/// A swap file can be recovered if:
/// - It has valid block 0 identification
/// - The strings in block 0 are valid (NUL-terminated)
/// - The byte order magic numbers are correct
///
/// # Safety
/// - `b0` must be a valid pointer to a ZeroBlock
#[no_mangle]
pub unsafe extern "C" fn rs_swap_file_recoverable(b0: *const c_void) -> c_int {
    if b0.is_null() {
        return 0;
    }

    // Check ID bytes
    if ml_check_b0_id_c(b0) != 0 {
        return 0;
    }

    // Check strings are valid
    if ml_check_b0_strings_c(b0) != 0 {
        return 0;
    }

    // Check byte order
    if b0_magic_wrong_c(b0) != 0 {
        return 0;
    }

    1
}

/// Get the swap file attention choice from a user response.
///
/// Maps user input characters to SEA_CHOICE constants:
/// - 'O' or 'o' -> SEA_CHOICE_READONLY (1)
/// - 'E' or 'e' -> SEA_CHOICE_EDIT (2)
/// - 'R' or 'r' -> SEA_CHOICE_RECOVER (3)
/// - 'D' or 'd' -> SEA_CHOICE_DELETE (4)
/// - 'Q' or 'q' -> SEA_CHOICE_QUIT (5)
/// - 'A' or 'a' -> SEA_CHOICE_ABORT (6)
/// - Other -> SEA_CHOICE_NONE (0)
#[no_mangle]
#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)] // Intentional: interpreting c_int as ASCII char code
pub extern "C" fn rs_sea_choice_from_char(c: c_int) -> c_int {
    match c as u8 {
        b'O' | b'o' => SEA_CHOICE_READONLY,
        b'E' | b'e' => SEA_CHOICE_EDIT,
        b'R' | b'r' => SEA_CHOICE_RECOVER,
        b'D' | b'd' => SEA_CHOICE_DELETE,
        b'Q' | b'q' => SEA_CHOICE_QUIT,
        b'A' | b'a' => SEA_CHOICE_ABORT,
        _ => SEA_CHOICE_NONE,
    }
}

/// Get a descriptive string for a SEA_CHOICE value.
///
/// Returns a static string describing the choice, or NULL for invalid values.
///
/// # Safety
/// Returns a pointer to static string data.
#[no_mangle]
pub extern "C" fn rs_sea_choice_name(choice: c_int) -> *const c_char {
    // Static strings for each choice
    static NONE: &[u8] = b"None\0";
    static READONLY: &[u8] = b"Open Read-Only\0";
    static EDIT: &[u8] = b"Edit anyway\0";
    static RECOVER: &[u8] = b"Recover\0";
    static DELETE: &[u8] = b"Delete it\0";
    static QUIT: &[u8] = b"Quit\0";
    static ABORT: &[u8] = b"Abort\0";

    match choice {
        x if x == SEA_CHOICE_NONE => NONE.as_ptr().cast(),
        x if x == SEA_CHOICE_READONLY => READONLY.as_ptr().cast(),
        x if x == SEA_CHOICE_EDIT => EDIT.as_ptr().cast(),
        x if x == SEA_CHOICE_RECOVER => RECOVER.as_ptr().cast(),
        x if x == SEA_CHOICE_DELETE => DELETE.as_ptr().cast(),
        x if x == SEA_CHOICE_QUIT => QUIT.as_ptr().cast(),
        x if x == SEA_CHOICE_ABORT => ABORT.as_ptr().cast(),
        _ => std::ptr::null(),
    }
}

/// Check if a SEA_CHOICE value means "proceed with editing".
///
/// Returns true for: EDIT, RECOVER
#[no_mangle]
pub extern "C" fn rs_sea_choice_proceeds(choice: c_int) -> c_int {
    c_int::from(choice == SEA_CHOICE_EDIT || choice == SEA_CHOICE_RECOVER)
}

/// Check if a SEA_CHOICE value means "abort operation".
///
/// Returns true for: QUIT, ABORT
#[no_mangle]
pub extern "C" fn rs_sea_choice_aborts(choice: c_int) -> c_int {
    c_int::from(choice == SEA_CHOICE_QUIT || choice == SEA_CHOICE_ABORT)
}

// =============================================================================
// Byte Order Conversion
// =============================================================================

/// Portable conversion of long to byte array.
///
/// Stores the value in a byte order independent format.
/// This is used for swap file portability across different architectures.
///
/// # Implementation
///
/// Each byte stores 8 bits of the value, starting with the least significant.
///
/// # Safety
/// - `s` must be a valid pointer to at least 8 bytes
#[no_mangle]
#[allow(clippy::cast_possible_wrap)] // Intentional: storing bytes
pub unsafe extern "C" fn rs_long_to_char_portable(n: i64, s: *mut c_char) {
    if s.is_null() {
        return;
    }

    let mut val = n;
    for i in 0..8 {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let byte = (val & 0xFF) as c_char;
        *s.add(i) = byte;
        val >>= 8;
    }
}

/// Portable conversion of byte array to long.
///
/// Reverses the conversion done by `rs_long_to_char_portable`.
///
/// # Safety
/// - `s` must be a valid pointer to at least 8 bytes
#[no_mangle]
#[allow(clippy::cast_sign_loss)] // Intentional: reading bytes
pub unsafe extern "C" fn rs_char_to_long_portable(s: *const c_char) -> i64 {
    if s.is_null() {
        return 0;
    }

    let mut result: i64 = 0;
    for i in (0..8).rev() {
        let byte = i64::from(*s.add(i) as u8);
        result = (result << 8) | byte;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sea_choice_constants() {
        // Verify SEA_CHOICE constants are accessible
        assert_eq!(SEA_CHOICE_NONE, 0);
        assert_eq!(SEA_CHOICE_READONLY, 1);
        assert_eq!(SEA_CHOICE_EDIT, 2);
        assert_eq!(SEA_CHOICE_RECOVER, 3);
        assert_eq!(SEA_CHOICE_DELETE, 4);
        assert_eq!(SEA_CHOICE_QUIT, 5);
        assert_eq!(SEA_CHOICE_ABORT, 6);
    }

    #[test]
    fn test_sea_choice_from_char() {
        assert_eq!(rs_sea_choice_from_char(c_int::from(b'O')), SEA_CHOICE_READONLY);
        assert_eq!(rs_sea_choice_from_char(c_int::from(b'o')), SEA_CHOICE_READONLY);
        assert_eq!(rs_sea_choice_from_char(c_int::from(b'E')), SEA_CHOICE_EDIT);
        assert_eq!(rs_sea_choice_from_char(c_int::from(b'R')), SEA_CHOICE_RECOVER);
        assert_eq!(rs_sea_choice_from_char(c_int::from(b'D')), SEA_CHOICE_DELETE);
        assert_eq!(rs_sea_choice_from_char(c_int::from(b'Q')), SEA_CHOICE_QUIT);
        assert_eq!(rs_sea_choice_from_char(c_int::from(b'A')), SEA_CHOICE_ABORT);
        assert_eq!(rs_sea_choice_from_char(c_int::from(b'X')), SEA_CHOICE_NONE);
    }

    #[test]
    fn test_sea_choice_proceeds() {
        assert_eq!(rs_sea_choice_proceeds(SEA_CHOICE_NONE), 0);
        assert_eq!(rs_sea_choice_proceeds(SEA_CHOICE_READONLY), 0);
        assert_eq!(rs_sea_choice_proceeds(SEA_CHOICE_EDIT), 1);
        assert_eq!(rs_sea_choice_proceeds(SEA_CHOICE_RECOVER), 1);
        assert_eq!(rs_sea_choice_proceeds(SEA_CHOICE_DELETE), 0);
        assert_eq!(rs_sea_choice_proceeds(SEA_CHOICE_QUIT), 0);
        assert_eq!(rs_sea_choice_proceeds(SEA_CHOICE_ABORT), 0);
    }

    #[test]
    fn test_sea_choice_aborts() {
        assert_eq!(rs_sea_choice_aborts(SEA_CHOICE_NONE), 0);
        assert_eq!(rs_sea_choice_aborts(SEA_CHOICE_READONLY), 0);
        assert_eq!(rs_sea_choice_aborts(SEA_CHOICE_EDIT), 0);
        assert_eq!(rs_sea_choice_aborts(SEA_CHOICE_RECOVER), 0);
        assert_eq!(rs_sea_choice_aborts(SEA_CHOICE_DELETE), 0);
        assert_eq!(rs_sea_choice_aborts(SEA_CHOICE_QUIT), 1);
        assert_eq!(rs_sea_choice_aborts(SEA_CHOICE_ABORT), 1);
    }

    #[test]
    fn test_long_char_conversion() {
        let mut buf = [0i8; 8];

        unsafe {
            // Test positive value
            rs_long_to_char_portable(0x1234_5678_9ABC_DEF0, buf.as_mut_ptr());
            let result = rs_char_to_long_portable(buf.as_ptr());
            assert_eq!(result, 0x1234_5678_9ABC_DEF0);

            // Test negative value
            rs_long_to_char_portable(-1, buf.as_mut_ptr());
            let result = rs_char_to_long_portable(buf.as_ptr());
            assert_eq!(result, -1);

            // Test zero
            rs_long_to_char_portable(0, buf.as_mut_ptr());
            let result = rs_char_to_long_portable(buf.as_ptr());
            assert_eq!(result, 0);
        }
    }
}
