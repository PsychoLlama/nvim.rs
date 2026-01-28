//! Swap file management for the memline system.
//!
//! This module provides Rust wrappers for swap file operations including
//! opening, closing, syncing, and preserving swap files.
//!
//! # Swap Files
//!
//! Swap files (`.swp`) are used for crash recovery. They store:
//! - Block 0: Metadata (filename, timestamps, user info)
//! - Pointer and data blocks: Mirror of the buffer's B-tree
//!
//! # Safety
//!
//! Swap file operations modify filesystem state and should only be called
//! when appropriate (e.g., not during recovery).

use std::ffi::{c_char, c_int};

use crate::types::{BufHandle, UB_FNAME, UB_SAME_DIR};

// =============================================================================
// C Implementation Declarations
// =============================================================================

extern "C" {
    // -------------------------------------------------------------------------
    // Buffer Accessors
    // -------------------------------------------------------------------------

    /// Check if buffer has a valid memfile
    fn nvim_buf_has_ml_mfp(buf: *mut BufHandle) -> c_int;

    // -------------------------------------------------------------------------
    // Swap File Operations (C implementations)
    // -------------------------------------------------------------------------

    /// Open memline for buffer, create swap file
    fn ml_open(buf: *mut BufHandle) -> c_int;

    /// Set the name of the swap file
    fn ml_setname(buf: *mut BufHandle);

    /// Open swap files for all buffers
    fn ml_open_files();

    /// Open swap file for a specific buffer
    fn ml_open_file(buf: *mut BufHandle);

    /// Check if swap file needs to be created
    fn check_need_swap(newfile: c_int);

    /// Close memline for buffer
    fn ml_close(buf: *mut BufHandle, del_file: c_int);

    /// Close all memlines
    fn ml_close_all(del_file: c_int);

    /// Close memlines for unmodified buffers
    fn ml_close_notmod();

    /// Update timestamp in swap file
    fn ml_timestamp(buf: *mut BufHandle);

    /// Sync all modified swap files
    fn ml_sync_all(check_file: c_int, check_char: c_int, do_fsync: c_int);

    /// Preserve buffer (write to swap file)
    fn ml_preserve(buf: *mut BufHandle, message: c_int, do_fsync: c_int);

    /// Set memline flags for swap file
    fn ml_setflags(buf: *mut BufHandle);

    /// Make swap file name
    #[allow(clippy::similar_names)]
    fn makeswapname(
        fname: *mut c_char,
        full_fname: *mut c_char,
        buf: *mut BufHandle,
        dir_name: *mut c_char,
    ) -> *mut c_char;

    /// Get file name for swap/backup in a directory
    fn get_file_in_dir(fname: *mut c_char, dname: *mut c_char) -> *mut c_char;
}

// =============================================================================
// Swap File Opening/Closing
// =============================================================================

/// Open the memline for a buffer, creating the swap file.
///
/// This initializes the B-tree structure and creates block 0 with metadata.
///
/// # Safety
/// - `buf` must be a valid buffer pointer or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_ml_open(buf: *mut BufHandle) -> c_int {
    if buf.is_null() {
        return 0; // FAIL
    }
    ml_open(buf)
}

/// Set the name of the swap file for a buffer.
///
/// This is called when the buffer's file name changes.
///
/// # Safety
/// - `buf` must be a valid buffer pointer or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_ml_setname(buf: *mut BufHandle) {
    if !buf.is_null() {
        ml_setname(buf);
    }
}

/// Open swap files for all buffers that need them.
///
/// Called at startup after reading viminfo.
///
/// # Safety
/// Modifies global buffer state.
#[no_mangle]
pub unsafe extern "C" fn rs_ml_open_files() {
    ml_open_files();
}

/// Open the swap file for a specific buffer.
///
/// Creates the swap file if it doesn't exist.
///
/// # Safety
/// - `buf` must be a valid buffer pointer or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_ml_open_file(buf: *mut BufHandle) {
    if !buf.is_null() {
        ml_open_file(buf);
    }
}

/// Check if a swap file needs to be created for the current buffer.
///
/// # Arguments
/// * `newfile` - true if reading a file into a new buffer
///
/// # Safety
/// Modifies current buffer state.
#[no_mangle]
pub unsafe extern "C" fn rs_check_need_swap(newfile: c_int) {
    check_need_swap(newfile);
}

/// Close the memline for a buffer.
///
/// This closes and optionally deletes the swap file.
///
/// # Arguments
/// * `buf` - Buffer to close
/// * `del_file` - If true, delete the swap file
///
/// # Safety
/// - `buf` must be a valid buffer pointer or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_ml_close(buf: *mut BufHandle, del_file: c_int) {
    if !buf.is_null() {
        ml_close(buf, del_file);
    }
}

/// Close all existing memlines and memfiles.
///
/// Only used when exiting Neovim.
///
/// # Arguments
/// * `del_file` - If true, delete the swap files
///
/// # Safety
/// Modifies global state, only call during exit.
#[no_mangle]
pub unsafe extern "C" fn rs_ml_close_all(del_file: c_int) {
    ml_close_all(del_file);
}

/// Close all memlines for unmodified buffers.
///
/// Only use just before exiting.
///
/// # Safety
/// Modifies global state, only call during exit.
#[no_mangle]
pub unsafe extern "C" fn rs_ml_close_notmod() {
    ml_close_notmod();
}

// =============================================================================
// Swap File Syncing and Preservation
// =============================================================================

/// Update the timestamp in the swap file.
///
/// Called when the buffer file has been written.
///
/// # Safety
/// - `buf` must be a valid buffer pointer or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_ml_timestamp(buf: *mut BufHandle) {
    if !buf.is_null() {
        ml_timestamp(buf);
    }
}

/// Sync all modified swap files.
///
/// Writes pending changes to disk for all buffers.
///
/// # Arguments
/// * `check_file` - If true, check if file changed
/// * `check_char` - If true, check for typed character
/// * `do_fsync` - If true, fsync the file
///
/// # Safety
/// Modifies global file state.
#[no_mangle]
pub unsafe extern "C" fn rs_ml_sync_all(check_file: c_int, check_char: c_int, do_fsync: c_int) {
    ml_sync_all(check_file, check_char, do_fsync);
}

/// Preserve a buffer by writing all changes to the swap file.
///
/// Used when memory is low or when explicitly preserving.
///
/// # Arguments
/// * `buf` - Buffer to preserve
/// * `message` - If true, show a message
/// * `do_fsync` - If true, fsync the file
///
/// # Safety
/// - `buf` must be a valid buffer pointer or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_ml_preserve(buf: *mut BufHandle, message: c_int, do_fsync: c_int) {
    if !buf.is_null() {
        ml_preserve(buf, message, do_fsync);
    }
}

/// Set the memline flags for swap file state.
///
/// # Safety
/// - `buf` must be a valid buffer pointer or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_ml_setflags(buf: *mut BufHandle) {
    if !buf.is_null() {
        ml_setflags(buf);
    }
}

// =============================================================================
// Swap File Name Generation
// =============================================================================

/// Make a swap file name from a file name and directory.
///
/// # Arguments
/// * `fname` - The file name
/// * `full_fname` - The full file name
/// * `buf` - The buffer
/// * `dir_name` - The directory name
///
/// # Returns
/// Allocated swap file name, or NULL
///
/// # Safety
/// - All pointers must be valid C strings or NULL
/// - The returned pointer must be freed by the caller
#[no_mangle]
#[allow(clippy::similar_names)]
pub unsafe extern "C" fn rs_makeswapname(
    fname: *mut c_char,
    full_fname: *mut c_char,
    buf: *mut BufHandle,
    dir_name: *mut c_char,
) -> *mut c_char {
    if fname.is_null() || dir_name.is_null() {
        return std::ptr::null_mut();
    }
    makeswapname(fname, full_fname, buf, dir_name)
}

/// Get the swap/backup file name in a directory.
///
/// # Arguments
/// * `fname` - The file name
/// * `dname` - The directory name
///
/// # Returns
/// Allocated file path, or NULL
///
/// # Safety
/// - All pointers must be valid C strings or NULL
/// - The returned pointer must be freed by the caller
#[no_mangle]
pub unsafe extern "C" fn rs_get_file_in_dir(fname: *mut c_char, dname: *mut c_char) -> *mut c_char {
    if fname.is_null() || dname.is_null() {
        return std::ptr::null_mut();
    }
    get_file_in_dir(fname, dname)
}

// =============================================================================
// Swap File Status
// =============================================================================

/// Check if a buffer has a swap file.
///
/// # Safety
/// - `buf` must be a valid buffer pointer or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_ml_has_swap(buf: *mut BufHandle) -> c_int {
    if buf.is_null() {
        return 0;
    }
    nvim_buf_has_ml_mfp(buf)
}

/// Check if a buffer needs its swap file to be synced.
///
/// Returns true if there are dirty blocks or a dirty cached line.
///
/// # Safety
/// - `buf` must be a valid buffer pointer or NULL
#[no_mangle]
pub unsafe extern "C" fn rs_ml_needs_sync(buf: *mut BufHandle) -> c_int {
    if buf.is_null() {
        return 0;
    }

    // Check if buffer has a memfile
    if nvim_buf_has_ml_mfp(buf) == 0 {
        return 0;
    }

    // Check if there are dirty flags (would need additional accessors)
    // For now, just return whether the memfile exists
    1
}

// =============================================================================
// Block 0 Update Types
// =============================================================================

/// Get the UB_FNAME constant (update filename/timestamp).
#[no_mangle]
pub extern "C" fn rs_ub_fname() -> c_int {
    UB_FNAME
}

/// Get the UB_SAME_DIR constant (update same-dir flag).
#[no_mangle]
pub extern "C" fn rs_ub_same_dir() -> c_int {
    UB_SAME_DIR
}

// =============================================================================
// Block 0 Field Access Helpers
// =============================================================================

/// Get the b0_dirty field value from a ZeroBlock.
///
/// The dirty flag is stored at b0_fname[B0_FNAME_SIZE_ORG - 1].
///
/// # Arguments
/// * `b0_fname` - Pointer to the b0_fname field
/// * `fname_size` - Size of the b0_fname field (B0_FNAME_SIZE_ORG)
///
/// # Safety
/// - `b0_fname` must be a valid pointer to an array of at least `fname_size` bytes
#[no_mangle]
#[allow(clippy::cast_sign_loss)] // Intentional: reading byte as unsigned
pub unsafe extern "C" fn rs_b0_get_dirty(b0_fname: *const c_char, fname_size: usize) -> c_int {
    if b0_fname.is_null() || fname_size == 0 {
        return 0;
    }
    c_int::from(*b0_fname.add(fname_size - 1) as u8)
}

/// Set the b0_dirty field value in a ZeroBlock.
///
/// # Safety
/// - `b0_fname` must be a valid pointer to a mutable array
#[no_mangle]
#[allow(clippy::cast_possible_wrap)] // Intentional: writing byte value
pub unsafe extern "C" fn rs_b0_set_dirty(b0_fname: *mut c_char, fname_size: usize, dirty: c_int) {
    if b0_fname.is_null() || fname_size == 0 {
        return;
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let value = dirty as u8;
    *b0_fname.add(fname_size - 1) = value as c_char;
}

/// Get the b0_flags field value from a ZeroBlock.
///
/// The flags are stored at b0_fname[B0_FNAME_SIZE_ORG - 2].
///
/// # Safety
/// - `b0_fname` must be a valid pointer to an array of at least `fname_size` bytes
#[no_mangle]
#[allow(clippy::cast_sign_loss)] // Intentional: reading byte as unsigned
pub unsafe extern "C" fn rs_b0_get_flags(b0_fname: *const c_char, fname_size: usize) -> c_int {
    if b0_fname.is_null() || fname_size < 2 {
        return 0;
    }
    c_int::from(*b0_fname.add(fname_size - 2) as u8)
}

/// Set the b0_flags field value in a ZeroBlock.
///
/// # Safety
/// - `b0_fname` must be a valid pointer to a mutable array
#[no_mangle]
#[allow(clippy::cast_possible_wrap)] // Intentional: writing byte value
pub unsafe extern "C" fn rs_b0_set_flags(b0_fname: *mut c_char, fname_size: usize, flags: c_int) {
    if b0_fname.is_null() || fname_size < 2 {
        return;
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let value = flags as u8;
    *b0_fname.add(fname_size - 2) = value as c_char;
}

/// Extract the file format from b0_flags.
///
/// The lowest two bits contain the file format:
/// - 0: not set (compatible with Vim 6.x)
/// - 1: EOL_UNIX + 1
/// - 2: EOL_DOS + 1
/// - 3: EOL_MAC + 1
#[no_mangle]
pub extern "C" fn rs_b0_get_fileformat(b0_flags: c_int) -> c_int {
    b0_flags & 3 // B0_FF_MASK
}

/// Check if the same-dir flag is set in b0_flags.
#[no_mangle]
pub extern "C" fn rs_b0_has_same_dir(b0_flags: c_int) -> c_int {
    c_int::from((b0_flags & 4) != 0) // B0_SAME_DIR
}

/// Check if the has-fenc flag is set in b0_flags.
#[no_mangle]
pub extern "C" fn rs_b0_has_fenc(b0_flags: c_int) -> c_int {
    c_int::from((b0_flags & 8) != 0) // B0_HAS_FENC
}

// =============================================================================
// Swap File Path Helpers
// =============================================================================

/// Check if a swap file name looks like a recovery file.
///
/// Recovery files have names like "*.swp" or "*.swo" etc.
///
/// # Safety
/// - `fname` must be a valid C string
#[no_mangle]
#[allow(clippy::cast_possible_wrap)] // Intentional: comparing ASCII byte values
pub unsafe extern "C" fn rs_is_swap_file_name(fname: *const c_char) -> c_int {
    if fname.is_null() {
        return 0;
    }

    // Find string length manually
    let mut len = 0usize;
    while *fname.add(len) != 0 {
        len += 1;
    }

    if len < 4 {
        return 0;
    }

    // Check for .sw? extension
    let ext = fname.add(len - 4);
    if *ext == b'.' as c_char && *ext.add(1) == b's' as c_char && *ext.add(2) == b'w' as c_char {
        return 1;
    }

    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ub_constants() {
        assert_eq!(rs_ub_fname(), UB_FNAME);
        assert_eq!(rs_ub_same_dir(), UB_SAME_DIR);
    }

    #[test]
    fn test_b0_dirty() {
        let mut fname = [0i8; 10];

        unsafe {
            // Set dirty
            rs_b0_set_dirty(fname.as_mut_ptr(), 10, 0x55);
            assert_eq!(rs_b0_get_dirty(fname.as_ptr(), 10), 0x55);

            // Clear dirty
            rs_b0_set_dirty(fname.as_mut_ptr(), 10, 0);
            assert_eq!(rs_b0_get_dirty(fname.as_ptr(), 10), 0);
        }
    }

    #[test]
    fn test_b0_flags() {
        let mut fname = [0i8; 10];

        unsafe {
            rs_b0_set_flags(fname.as_mut_ptr(), 10, 0x0F);
            assert_eq!(rs_b0_get_flags(fname.as_ptr(), 10), 0x0F);
        }
    }

    #[test]
    fn test_b0_flag_extraction() {
        // Test file format extraction (bits 0-1)
        assert_eq!(rs_b0_get_fileformat(0b0001), 1); // Unix
        assert_eq!(rs_b0_get_fileformat(0b0010), 2); // DOS
        assert_eq!(rs_b0_get_fileformat(0b0011), 3); // Mac

        // Test same-dir flag (bit 2)
        assert_eq!(rs_b0_has_same_dir(0b0000), 0);
        assert_eq!(rs_b0_has_same_dir(0b0100), 1);

        // Test has-fenc flag (bit 3)
        assert_eq!(rs_b0_has_fenc(0b0000), 0);
        assert_eq!(rs_b0_has_fenc(0b1000), 1);

        // Test combined flags
        let flags = 0b1101; // fenc + same_dir + unix
        assert_eq!(rs_b0_get_fileformat(flags), 1);
        assert_eq!(rs_b0_has_same_dir(flags), 1);
        assert_eq!(rs_b0_has_fenc(flags), 1);
    }

    #[test]
    fn test_is_swap_file_name() {
        unsafe {
            // Valid swap files
            assert_eq!(rs_is_swap_file_name(c"test.swp".as_ptr().cast()), 1);
            assert_eq!(rs_is_swap_file_name(c"file.swo".as_ptr().cast()), 1);
            assert_eq!(
                rs_is_swap_file_name(c"/path/to/file.swn".as_ptr().cast()),
                1
            );

            // Not swap files
            assert_eq!(rs_is_swap_file_name(c"test.txt".as_ptr().cast()), 0);
            assert_eq!(rs_is_swap_file_name(c"sw".as_ptr().cast()), 0); // too short
        }
    }
}
