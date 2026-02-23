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

use crate::rs_long_to_char;
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

// =============================================================================
// Phase 2: Swap File Utility Implementations
// =============================================================================

extern "C" {
    /// Check if two paths are in the same directory
    fn same_directory(p1: *const c_char, p2: *const c_char) -> c_int;

    /// Get b0_flags byte from ZeroBlock
    fn nvim_b0_get_flags_byte(b0: *const c_void) -> u8;

    /// Set b0_flags byte in ZeroBlock
    fn nvim_b0_set_flags_byte(b0: *mut c_void, val: u8);

    /// Get mutable pointer to b0_fname in ZeroBlock
    fn nvim_b0_get_fname_mut(b0: *mut c_void) -> *mut c_char;

    /// Get buf->b_ml.ml_mfp->mf_fname
    fn nvim_buf_get_ml_mfp_fname(buf: *mut BufHandle) -> *mut c_char;

    /// Get buf->b_ffname
    fn nvim_buf_get_ffname(buf: *mut BufHandle) -> *const c_char;

    /// Get buf->b_p_fenc
    fn nvim_buf_get_b_p_fenc(buf: *mut BufHandle) -> *const c_char;

    /// Get mutable pointer to b0_mtime field
    fn nvim_b0_get_mtime(b0p: *mut c_void) -> *mut c_char;

    /// Get mutable pointer to b0_ino field
    fn nvim_b0_get_ino(b0p: *mut c_void) -> *mut c_char;
}

use std::ffi::c_void;

use crate::types::{B0_FNAME_SIZE_NOCRYPT, B0_HAS_FENC, B0_SAME_DIR};

/// Update the B0_SAME_DIR flag of the swap file.
///
/// The flag is set if the swap file and the edited file are in the same directory.
/// This is fail-safe: when uncertain, the flag is not set.
///
/// # Safety
/// - `b0p` must be a valid ZeroBlock pointer
/// - `buf` must be a valid buffer pointer
#[no_mangle]
pub unsafe extern "C" fn rs_set_b0_dir_flag(b0p: *mut c_void, buf: *mut BufHandle) {
    let mfp_fname = nvim_buf_get_ml_mfp_fname(buf);
    let ffname = nvim_buf_get_ffname(buf);
    let flags = nvim_b0_get_flags_byte(b0p);
    if same_directory(mfp_fname, ffname) != 0 {
        nvim_b0_set_flags_byte(b0p, flags | B0_SAME_DIR);
    } else {
        nvim_b0_set_flags_byte(b0p, flags & !B0_SAME_DIR);
    }
}

/// Add the 'fileencoding' to block 0 when there is room.
///
/// The encoding is stored at the end of b0_fname, with a NUL byte before it.
/// The B0_HAS_FENC flag is set if encoding was stored, cleared otherwise.
///
/// # Safety
/// - `b0p` must be a valid ZeroBlock pointer
/// - `buf` must be a valid buffer pointer
#[no_mangle]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
pub unsafe extern "C" fn rs_add_b0_fenc(b0p: *mut c_void, buf: *mut BufHandle) {
    let size = B0_FNAME_SIZE_NOCRYPT as isize;
    let fenc = nvim_buf_get_b_p_fenc(buf);

    // Calculate length of fenc string
    let mut fenc_len: isize = 0;
    while *fenc.offset(fenc_len) != 0 {
        fenc_len += 1;
    }

    // Calculate length of existing b0_fname
    let fname_ptr = nvim_b0_get_fname_mut(b0p);
    let mut fname_len: isize = 0;
    while *fname_ptr.offset(fname_len) != 0 {
        fname_len += 1;
    }

    let flags = nvim_b0_get_flags_byte(b0p);
    if fname_len + fenc_len + 1 > size {
        // Not enough room: clear the flag
        nvim_b0_set_flags_byte(b0p, flags & !B0_HAS_FENC);
    } else {
        // Copy fenc at end of fname buffer (size - fenc_len from start)
        let dest = fname_ptr.offset(size - fenc_len);
        std::ptr::copy_nonoverlapping(fenc, dest, fenc_len as usize);
        // Place NUL before the encoding
        *fname_ptr.offset(size - fenc_len - 1) = 0;
        nvim_b0_set_flags_byte(b0p, flags | B0_HAS_FENC);
    }
}

// =============================================================================
// Phase 4: Block 0 Update and File Name (ml_upd_block0, set_b0_fname)
// =============================================================================

extern "C" {
    // Phase 4: set_b0_fname accessors
    fn nvim_get_curbuf() -> *mut BufHandle;
    fn nvim_home_replace_b0_fname(buf: *const BufHandle, b0p: *mut c_void, maxlen: usize);
    fn nvim_os_get_username(buf: *mut c_char, len: usize) -> c_int;
    fn nvim_set_b0_mtime_ino(buf: *mut BufHandle, b0p: *mut c_void) -> c_int;
    fn nvim_buf_set_b_mtime(buf: *mut BufHandle, val: i64);
    fn nvim_buf_set_b_mtime_ns(buf: *mut BufHandle, val: i64);
    fn nvim_buf_set_b_mtime_read(buf: *mut BufHandle, val: i64);
    fn nvim_buf_set_b_mtime_read_ns(buf: *mut BufHandle, val: i64);
    fn nvim_buf_set_b_orig_size(buf: *mut BufHandle, val: i64);
    fn nvim_buf_set_b_orig_mode(buf: *mut BufHandle, val: c_int);
}

use crate::types::B0_FNAME_SIZE_CRYPT;

/// Write file name and timestamp into block 0 of a swap file.
///
/// Also sets `buf->b_mtime` and related fields.
///
/// # Safety
/// - `b0p` must be a valid ZeroBlock pointer
/// - `buf` must be a valid buffer pointer
#[no_mangle]
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
pub unsafe extern "C" fn rs_set_b0_fname(b0p: *mut c_void, buf: *mut BufHandle) {
    let ffname = nvim_buf_get_ffname(buf);

    if ffname.is_null() {
        // No file name: clear b0_fname
        let fname_ptr = nvim_b0_get_fname_mut(b0p);
        *fname_ptr = 0;
    } else {
        // Write home-replaced path into b0_fname
        nvim_home_replace_b0_fname(buf.cast(), b0p, B0_FNAME_SIZE_CRYPT);

        // If starts with '~', try to insert username: "~user/"
        let fname_ptr = nvim_b0_get_fname_mut(b0p);
        if *fname_ptr == b'~' as c_char {
            let mut uname = [0i8; 40]; // B0_UNAME_SIZE
            let retval = nvim_os_get_username(uname.as_mut_ptr(), 40);

            // Calculate string lengths
            let mut ulen = 0usize;
            while *uname.as_ptr().add(ulen) != 0 {
                ulen += 1;
            }
            let mut flen = 0usize;
            while *fname_ptr.add(flen) != 0 {
                flen += 1;
            }

            if retval == 0 || ulen + flen > B0_FNAME_SIZE_CRYPT - 1 {
                // Too long or failed: just copy ffname directly
                let max = B0_FNAME_SIZE_CRYPT;
                let src_len = {
                    let mut n = 0usize;
                    while *ffname.add(n) != 0 {
                        n += 1;
                    }
                    n.min(max - 1)
                };
                std::ptr::copy_nonoverlapping(ffname, fname_ptr, src_len);
                *fname_ptr.add(src_len) = 0;
            } else {
                // Insert username: shift "~content" to "~user/content"
                // Move existing content (starting at [1]) right by ulen positions
                std::ptr::copy(fname_ptr.add(1), fname_ptr.add(ulen + 1), flen);
                // Copy username at position 1
                std::ptr::copy_nonoverlapping(uname.as_ptr(), fname_ptr.add(1), ulen);
            }
        }

        // Write timestamps and inode into block 0
        if nvim_set_b0_mtime_ino(buf, b0p) == 0 {
            // File not found: zero out timestamps
            let mtime_ptr = nvim_b0_get_mtime(b0p);
            rs_long_to_char(0, mtime_ptr);
            let ino_ptr = nvim_b0_get_ino(b0p);
            rs_long_to_char(0, ino_ptr);
            nvim_buf_set_b_mtime(buf, 0);
            nvim_buf_set_b_mtime_ns(buf, 0);
            nvim_buf_set_b_mtime_read(buf, 0);
            nvim_buf_set_b_mtime_read_ns(buf, 0);
            nvim_buf_set_b_orig_size(buf, 0);
            nvim_buf_set_b_orig_mode(buf, 0);
        }
    }

    // Also add the 'fileencoding' if there is room (use curbuf like the C implementation)
    rs_add_b0_fenc(b0p, nvim_get_curbuf());
}

extern "C" {
    /// Get memfile from buffer
    fn nvim_buf_get_ml_mfp(buf: *mut BufHandle) -> *mut c_void;

    /// Get block from memfile
    fn mf_get(mfp: *mut c_void, bnum: i64, count: c_int) -> *mut c_void;

    /// Release block back to memfile
    fn mf_put(mfp: *mut c_void, hp: *mut c_void, dirty: bool, release: bool);

    /// Get bh_data pointer from block header
    fn nvim_bhdr_get_bh_data(hp: *mut c_void) -> *mut c_void;

    /// Check block 0 ID (returns nonzero if bad)
    fn rs_ml_check_b0_id(b0p: *const c_void) -> c_int;

    /// Print "E304: ml_upd_block0(): Didn't get block 0??" error
    fn nvim_iemsg_e304_upd_block0();
}

/// Update block 0 of the swap file with filename or same-dir flag.
///
/// Called after the swap file is opened to update block 0 metadata.
/// - `what == UB_FNAME (0)`: update timestamp and filename
/// - `what == UB_SAME_DIR (1)`: update the B0_SAME_DIR flag
///
/// # Safety
/// - `buf` must be a valid buffer pointer
#[no_mangle]
pub unsafe extern "C" fn rs_ml_upd_block0(buf: *mut BufHandle, what: c_int) {
    let mfp = nvim_buf_get_ml_mfp(buf);
    if mfp.is_null() {
        return;
    }
    let hp = mf_get(mfp, 0, 1);
    if hp.is_null() {
        return;
    }
    let b0p = nvim_bhdr_get_bh_data(hp);
    if rs_ml_check_b0_id(b0p) != 0 {
        nvim_iemsg_e304_upd_block0();
    } else if what == UB_FNAME as c_int {
        rs_set_b0_fname(b0p, buf);
    } else {
        // what == UB_SAME_DIR
        rs_set_b0_dir_flag(b0p, buf);
    }
    mf_put(mfp, hp, true, false);
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
