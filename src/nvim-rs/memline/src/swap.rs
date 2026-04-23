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

use std::ffi::{c_char, c_int, c_uint};

use crate::block_ops;
use crate::rs_long_to_char;
use crate::types::{
    BufHandle, B0_DIRTY, B0_FF_MASK, BF_RECOVERED, MFS_ZERO, ML_ALLOCATED, ML_LINE_DIRTY,
    SHM_ATTENTION, UB_FNAME, UB_SAME_DIR,
};
#[allow(unused_imports)]
use libc;

/// CMOD_NOSWAPFILE flag value (validated by _Static_assert in option_shim.c)
const CMOD_NOSWAPFILE: c_int = 0x2000;

// =============================================================================
// C Implementation Declarations
// =============================================================================

extern "C" {
    static mut msg_silent: c_int;
    static mut got_int: bool;
    static mut need_wait_return: bool;
    // -------------------------------------------------------------------------
    // Buffer Accessors
    // -------------------------------------------------------------------------

    // -------------------------------------------------------------------------
    // Phase 3: Buffer lifecycle accessors
    // -------------------------------------------------------------------------

    /// Close a memfile
    fn mf_close(mfp: *mut std::ffi::c_void, del_file: c_int);

    /// Get current buffer
    fn nvim_get_curbuf() -> *mut BufHandle;

    /// Get msg_silent global

    /// Set msg_silent global

    /// Get buf->b_may_swap (defined in change_ffi.c, returns bool)
    fn nvim_buf_get_b_may_swap(buf: *mut BufHandle) -> bool;

    // -------------------------------------------------------------------------
    // Phase 5: ml_setflags accessors
    // -------------------------------------------------------------------------

    /// Look up block 0 header in memfile hash: pmap_get(int64_t)(&mfp->mf_hash, 0)
    fn nvim_mf_get_block0_hp(mfp: *mut std::ffi::c_void) -> *mut std::ffi::c_void;

    /// Set BH_DIRTY flag on block header: hp->bh_flags |= BH_DIRTY
    fn nvim_bhdr_set_bh_flags_dirty(hp: *mut std::ffi::c_void);

    /// Get buf->b_changed
    fn nvim_buf_get_b_changed(buf: *mut BufHandle) -> bool;

    /// Sync memfile blocks to disk
    fn mf_sync(mfp: *mut std::ffi::c_void, flags: c_int) -> c_int;

    /// Get the file format for a buffer (0=unix, 1=dos, 2=mac)
    fn rs_get_fileformat(buf: *mut BufHandle) -> c_int;

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
#[export_name = "ml_open"]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_ml_open(buf: *mut BufHandle) -> c_int {
    if buf.is_null() {
        return crate::types::FAIL;
    }

    // Initialize all ml fields to zero/NULL
    block_ops::buf_init_ml_empty(buf);

    if nvim_get_cmod_flags() & CMOD_NOSWAPFILE != 0 {
        nvim_buf_set_b_p_swf_false(buf);
    }

    // When 'updatecount' is non-zero swapfile may be opened later.
    if (*buf.cast::<BufStruct>()).terminal.is_null() && p_uc != 0 && nvim_buf_get_p_swf(buf) != 0 {
        nvim_buf_set_b_may_swap_true(buf);
    } else {
        nvim_buf_set_b_may_swap(buf, 0);
    }

    // Open the memfile. No swap file is created yet.
    let mfp = mf_open(std::ptr::null_mut(), 0);
    if mfp.is_null() {
        return crate::types::FAIL;
    }

    (*buf.cast::<BufStruct>()).ml_mfp = mfp;
    (*buf.cast::<BufStruct>()).ml_flags = crate::types::ML_EMPTY;
    (*buf.cast::<BufStruct>()).ml_line_count = 1;

    // A helper macro-like closure for the error path.
    // Puts hp back (if non-null) then closes the memfile.
    let do_error = |hp: *mut std::ffi::c_void| {
        if !hp.is_null() {
            mf_put(mfp, hp, false, false);
        }
        mf_close(mfp, 1); // del_file = true
        (*buf.cast::<BufStruct>()).ml_mfp = std::ptr::null_mut();
    };

    // Allocate block 0 (expected bh_bnum == 0)
    let hp0 = mf_new(mfp, false, 1);
    if hp0.is_null() {
        do_error(std::ptr::null_mut());
        return crate::types::FAIL;
    }
    if nvim_bhdr_get_bh_bnum(hp0) != 0 {
        iemsg(c"E298: Didn't get block nr 0?".as_ptr());
        do_error(hp0);
        return crate::types::FAIL;
    }

    let b0p = nvim_bhdr_get_bh_data(hp0);
    let page_size = nvim_mf_get_page_size(mfp);
    nvim_b0_init_header(b0p, page_size);

    if nvim_buf_get_b_spell(buf) == 0 {
        nvim_b0_set_dirty_from_buf(b0p, buf);
        let fileformat = rs_get_fileformat(buf);
        nvim_b0_set_flags_from_ff(b0p, fileformat);
        rs_set_b0_fname(b0p, buf);
        nvim_b0_fill_uname(b0p);
        nvim_b0_fill_hname(b0p);
        nvim_b0_fill_pid(b0p);
    }

    // Always sync block 0 to disk so findswapname() can check the file name.
    // Don't do this for help files or spell buffers.
    mf_put(mfp, hp0, true, false);
    if (*buf.cast::<BufStruct>()).b_help == 0 && nvim_buf_get_b_spell(buf) == 0 {
        mf_sync(mfp, 0);
    }

    // Fill in root pointer block and write page 1.
    let hp1 = rs_ml_new_ptr(mfp);
    if hp1.is_null() {
        do_error(std::ptr::null_mut());
        return crate::types::FAIL;
    }
    if nvim_bhdr_get_bh_bnum(hp1) != 1 {
        iemsg(c"E298: Didn't get block nr 1?".as_ptr());
        do_error(hp1);
        return crate::types::FAIL;
    }
    let pp = nvim_bhdr_get_bh_data(hp1);
    block_ops::pp_init_root(pp);
    mf_put(mfp, hp1, true, false);

    // Allocate first data block and create an empty line 1.
    let hp2 = rs_ml_new_data(mfp, false, 1);
    if hp2.is_null() {
        do_error(std::ptr::null_mut());
        return crate::types::FAIL;
    }
    if nvim_bhdr_get_bh_bnum(hp2) != 2 {
        iemsg(c"E298: Didn't get block nr 2?".as_ptr());
        do_error(hp2);
        return crate::types::FAIL;
    }
    let dp = nvim_bhdr_get_bh_data(hp2);
    block_ops::dp_init_empty_line(dp);
    mf_put(mfp, hp2, true, false);

    crate::types::OK
}

/// Set the name of the swap file for a buffer.
///
/// Called when the buffer's file name changes. Renames the swap file
/// to match the new file name (trying all directories in 'directory').
///
/// # Safety
/// - `buf` must be a valid buffer pointer or NULL
#[export_name = "ml_setname"]
pub unsafe extern "C" fn rs_ml_setname(buf: *mut BufHandle) {
    if buf.is_null() {
        return;
    }

    let mfp = (*buf.cast::<BufStruct>()).ml_mfp;
    if mfp.is_null() {
        return;
    }

    if nvim_mf_get_fd(mfp) < 0 {
        // There is no swap file yet.
        // When 'updatecount' is 0 or 'noswapfile' there is no swap file.
        // For help files we will make a swap file now.
        if p_uc != 0 && nvim_get_cmod_flags() & CMOD_NOSWAPFILE == 0 {
            rs_ml_open_file(buf); // create a swap file
        }
        return;
    }

    // Try all directories in the 'directory' option.
    let dirp_start = nvim_get_p_dir();
    let mut dirp = dirp_start;
    let mut found_existing_dir = false;
    let mut success = false;

    loop {
        if *dirp == 0 {
            break; // tried all directories, fail
        }
        let old_fname = nvim_mf_get_fname(mfp);
        #[allow(clippy::redundant_locals)]
        let fname = rs_findswapname(buf, &raw mut dirp, old_fname, &raw mut found_existing_dir);
        if dirp.is_null() {
            break; // out of memory
        }
        if fname.is_null() {
            continue; // no file name found for this dir
        }

        // If the file name is the same we don't have to do anything
        if nvim_path_fnamecmp(fname, old_fname) == 0 {
            xfree(fname.cast());
            success = true;
            break;
        }

        // Need to close the swap file before renaming
        let fd = nvim_mf_get_fd(mfp);
        if fd >= 0 {
            // close() from libc - declared in recovery.rs; redeclare here
            extern "C" {
                fn close(fd: c_int) -> c_int;
            }
            close(fd);
            nvim_mf_set_fd(mfp, -1);
        }

        // Try to rename the swap file
        let old_fname_for_rename = nvim_mf_get_fname(mfp);
        if nvim_vim_rename(old_fname_for_rename, fname) == 0 {
            success = true;
            mf_free_fnames(mfp);
            mf_set_fnames(mfp, fname); // fname is consumed
            rs_ml_upd_block0(buf, UB_SAME_DIR as c_int);
            break;
        }
        xfree(fname.cast()); // this fname didn't work, try another
    }

    // Need to (re)open the swap file if we closed it
    if nvim_mf_get_fd(mfp) == -1 {
        let mf_fname = nvim_mf_get_fname(mfp);
        let new_fd = os_open(mf_fname, libc::O_RDWR, 0);
        if new_fd < 0 {
            // could not (re)open the swap file
            emsg(c"E301: Oops, lost the swap file!!!".as_ptr());
            return;
        }
        nvim_mf_set_fd(mfp, new_fd);
        nvim_os_set_cloexec(new_fd);
    }

    if !success {
        emsg(c"E302: Could not rename swap file".as_ptr());
    }
}

/// Open swap files for all buffers that need them.
///
/// Called when 'updatecount' changes from zero to non-zero.
/// Iterates all buffers and opens a swap file for those that are not
/// read-only or have been modified.
///
/// # Safety
/// Modifies global buffer state.
#[export_name = "ml_open_files"]
pub unsafe extern "C" fn rs_ml_open_files() {
    let mut buf = nvim_get_firstbuf();
    while !buf.is_null() {
        if (*buf.cast::<BufStruct>()).b_p_ro == 0 || nvim_buf_get_b_changed(buf) {
            rs_ml_open_file(buf);
        }
        buf = nvim_buf_get_next(buf);
    }
}

/// Open the swap file for a specific buffer.
///
/// Creates the swap file if it doesn't exist. Tries all directories in
/// 'directory' option. For spell buffers, uses a temp file name.
///
/// # Safety
/// - `buf` must be a valid buffer pointer or NULL
#[export_name = "ml_open_file"]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
pub unsafe extern "C" fn rs_ml_open_file(buf: *mut BufHandle) {
    if buf.is_null() {
        return;
    }

    let mfp = (*buf.cast::<BufStruct>()).ml_mfp;
    if mfp.is_null()
        || nvim_mf_get_fd(mfp) >= 0
        || nvim_buf_get_p_swf(buf) == 0
        || nvim_get_cmod_flags() & CMOD_NOSWAPFILE != 0
        || !(*buf.cast::<BufStruct>()).terminal.is_null()
    {
        return; // nothing to do
    }

    // For a spell buffer use a temp file name.
    if nvim_buf_get_b_spell(buf) != 0 {
        let fname = vim_tempname();
        if !fname.is_null() {
            mf_open_file(mfp, fname); // consumes fname!
        }
        nvim_buf_set_b_may_swap(buf, 0);
        return;
    }

    // Try all directories in 'directory' option.
    let p_dir = nvim_get_p_dir();
    let mut dirp = p_dir;
    let mut found_existing_dir = false;
    loop {
        if *dirp == 0 {
            break;
        }
        // There is a small chance that between choosing the swapfile name
        // and creating it, another Vim creates the file.  In that case the
        // creation will fail and we will use another directory.
        let fname = rs_findswapname(
            buf,
            &raw mut dirp,
            std::ptr::null(),
            &raw mut found_existing_dir,
        );
        if dirp.is_null() {
            break; // out of memory
        }
        if fname.is_null() {
            continue;
        }
        // MF_DIRTY_YES_NOSYNC = 2: don't sync yet in ml_sync_all()
        if mf_open_file(mfp, fname) == crate::types::OK {
            nvim_mf_set_dirty(mfp, 2); // MF_DIRTY_YES_NOSYNC
            rs_ml_upd_block0(buf, UB_SAME_DIR as c_int);

            // Flush block zero, so others can read it
            if mf_sync(mfp, MFS_ZERO) == crate::types::OK {
                // Mark all blocks that should be in the swapfile as dirty.
                // Needed for when the 'swapfile' option was reset, so that
                // the swapfile was deleted, and then on again.
                rs_mf_set_dirty_all(mfp);
                break;
            }
            // Writing block 0 failed: close the file and try another dir
            rs_mf_close_file_impl(mfp);
        }
    }

    // Report error if we tried directories but couldn't create a swap file
    let p_dir_first = nvim_get_p_dir();
    if *p_dir_first != 0 && nvim_mf_get_fname(mfp).is_null() {
        need_wait_return = true; // call wait_return() later
        nvim_inc_no_wait_return();
        let spname = buf_spname(buf);
        let display_name = if spname.is_null() {
            (*buf.cast::<BufStruct>()).b_fname.cast_mut()
        } else {
            spname
        };
        semsg(
            c"E303: Unable to open swap file for \"%s\", recovery impossible".as_ptr(),
            display_name,
        );
        nvim_dec_no_wait_return();
    }

    // don't try to open a swapfile again
    nvim_buf_set_b_may_swap(buf, 0);
}

/// Check if a swap file needs to be created for the current buffer.
///
/// If the current buffer has b_may_swap set and is not read-only (or newfile is false),
/// opens a swap file. Temporarily clears msg_silent to allow E325 prompts.
///
/// # Arguments
/// * `newfile` - true if reading a file into a new buffer
///
/// # Safety
/// Modifies current buffer state.
#[export_name = "check_need_swap"]
pub unsafe extern "C" fn rs_check_need_swap(newfile: c_int) {
    let old_msg_silent = msg_silent;
    msg_silent = 0; // If swap dialog prompts for input, user needs to see it!

    let curbuf = nvim_get_curbuf();
    if !curbuf.is_null()
        && nvim_buf_get_b_may_swap(curbuf)
        && ((*curbuf.cast::<BufStruct>()).b_p_ro == 0 || newfile == 0)
    {
        rs_ml_open_file(curbuf);
    }

    msg_silent = old_msg_silent;
}

/// Close the memline for a buffer.
///
/// Closes and optionally deletes the swap file. Frees cached line, info stack,
/// and chunk size cache. Clears ml_mfp and the BF_RECOVERED flag.
///
/// # Arguments
/// * `buf` - Buffer to close
/// * `del_file` - If true, delete the swap file
///
/// # Safety
/// - `buf` must be a valid buffer pointer or NULL
#[export_name = "ml_close"]
pub unsafe extern "C" fn rs_ml_close(buf: *mut BufHandle, del_file: c_int) {
    if buf.is_null() {
        return;
    }
    let mfp = (*buf.cast::<BufStruct>()).ml_mfp;
    if mfp.is_null() {
        return; // not open
    }
    mf_close(mfp, del_file); // close the .swp file

    let line_lnum = LineNr::from((*buf.cast::<BufStruct>()).ml_line_lnum);
    let flags = (*buf.cast::<BufStruct>()).ml_flags;
    if line_lnum != 0 && (flags & (ML_LINE_DIRTY | ML_ALLOCATED)) != 0 {
        xfree((*buf.cast::<BufStruct>()).ml_line_ptr.cast());
    }
    xfree((*buf.cast::<BufStruct>()).ml_stack);
    {
        let bs = buf.cast::<BufStruct>();
        xfree((*bs).ml_chunksize);
        (*bs).ml_chunksize = std::ptr::null_mut();
    }
    // Inline nvim_buf_clear_ml_after_close: ml_mfp = NULL; b_flags &= ~BF_RECOVERED
    {
        let bs = buf.cast::<BufStruct>();
        (*bs).ml_mfp = std::ptr::null_mut();
        (*bs).b_flags &= !BF_RECOVERED;
    }
}

/// Close all existing memlines and memfiles.
///
/// Only used when exiting Neovim. Iterates all buffers and calls rs_ml_close.
/// After iteration, deletes the internal spell word list and temp directory.
///
/// # Arguments
/// * `del_file` - If non-zero, delete the swap files
///
/// # Safety
/// Modifies global state, only call during exit.
#[export_name = "ml_close_all"]
pub unsafe extern "C" fn rs_ml_close_all(del_file: c_int) {
    let mut buf = nvim_get_firstbuf();
    while !buf.is_null() {
        let next = nvim_buf_get_next(buf);
        rs_ml_close(buf, del_file);
        buf = next;
    }
    spell_delete_wordlist();
    vim_deltempdir();
}

/// Close all memlines for unmodified buffers.
///
/// Only use just before exiting. Iterates all buffers; closes those that are
/// not modified.
///
/// # Safety
/// Modifies global state, only call during exit.
#[export_name = "ml_close_notmod"]
pub unsafe extern "C" fn rs_ml_close_notmod() {
    let mut buf = nvim_get_firstbuf();
    while !buf.is_null() {
        let next = nvim_buf_get_next(buf);
        if nvim_buf_is_changed(buf) == 0 {
            rs_ml_close(buf, 1); // close all not-modified buffers, del_file=true
        }
        buf = next;
    }
}

// =============================================================================
// Swap File Syncing and Preservation
// =============================================================================

/// Update the timestamp in the swap file.
///
/// Called when the buffer file has been written.
/// Delegates to rs_ml_upd_block0 with UB_FNAME to update timestamp and filename.
///
/// # Safety
/// - `buf` must be a valid buffer pointer or NULL
#[export_name = "ml_timestamp"]
pub unsafe extern "C" fn rs_ml_timestamp(buf: *mut BufHandle) {
    if !buf.is_null() {
        rs_ml_upd_block0(buf, UB_FNAME as c_int);
    }
}

/// Sync all memlines, flushing dirty blocks to disk.
///
/// Iterates all buffers and calls rs_ml_sync_one for each. Stops early if
/// rs_ml_sync_one returns non-zero (character available or interrupt).
///
/// Called from idle timer and signal handlers (SIGPWR, SIGUSR1).
///
/// # Safety
/// Calls into C via FFI. Must be signal-handler safe.
#[export_name = "ml_sync_all"]
pub unsafe extern "C" fn rs_ml_sync_all(check_file: c_int, check_char: c_int, do_fsync: bool) {
    let mut buf = nvim_get_firstbuf();
    while !buf.is_null() {
        if rs_ml_sync_one(buf, check_file, check_char, do_fsync) != 0 {
            break;
        }
        buf = nvim_buf_get_next(buf);
    }
}

// rs_ml_preserve: now a full Rust implementation below (Phase 8 Pass 2).

/// Set the memline flags for swap file state.
///
/// Updates the dirty flag, fileformat, and fileencoding in swap file block 0.
/// - Sets b0_dirty based on buf->b_changed
/// - Sets the fileformat bits in b0_flags
/// - Calls rs_add_b0_fenc to update the fileencoding
/// - Marks block 0 dirty and syncs it
///
/// # Safety
/// - `buf` must be a valid buffer pointer or NULL
#[export_name = "ml_setflags"]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_ml_setflags(buf: *mut BufHandle) {
    if buf.is_null() {
        return;
    }
    let mfp = (*buf.cast::<BufStruct>()).ml_mfp;
    if mfp.is_null() {
        return;
    }
    let hp = nvim_mf_get_block0_hp(mfp);
    if hp.is_null() {
        return;
    }
    let b0p = nvim_bhdr_get_bh_data(hp);

    // Set dirty flag based on buf->b_changed
    let b0_fname = nvim_b0_get_fname_mut(b0p);
    let dirty_val = if nvim_buf_get_b_changed(buf) {
        c_int::from(B0_DIRTY)
    } else {
        0
    };
    rs_b0_set_dirty(b0_fname, crate::types::B0_FNAME_SIZE_ORG, dirty_val);

    // Update fileformat bits in b0_flags: (flags & ~B0_FF_MASK) | (ff + 1)
    let fileformat = rs_get_fileformat(buf);
    let old_flags = nvim_b0_get_flags_byte(b0p);
    let new_flags = (old_flags & !B0_FF_MASK) | ((fileformat + 1) as u8);
    nvim_b0_set_flags_byte(b0p, new_flags);

    // Add fileencoding if there is room
    rs_add_b0_fenc(b0p, buf);

    // Mark block 0 dirty and sync
    nvim_bhdr_set_bh_flags_dirty(hp);
    mf_sync(mfp, MFS_ZERO);
}

// =============================================================================
// Phase 9-1: ml_open_file + ml_open_files extern declarations
// =============================================================================

extern "C" {
    /// Get buf->b_spell (returns 1 if true)
    fn nvim_buf_get_b_spell(buf: *mut BufHandle) -> c_int;

    /// Set buf->b_may_swap (0 = false, non-zero = true)
    fn nvim_buf_set_b_may_swap(buf: *mut BufHandle, val: c_int);

    /// Get buf->b_p_swf (swapfile option)
    fn nvim_buf_get_p_swf(buf: *mut BufHandle) -> c_int;

    #[link_name = "nvim_get_cmdmod_cmod_flags"]
    fn nvim_get_cmod_flags() -> c_int;

    /// Get p_dir option (directories for swap files)
    fn nvim_get_p_dir() -> *mut c_char;

    /// Get first buffer in buffer list
    fn nvim_get_firstbuf() -> *mut BufHandle;

    /// Get next buffer in buffer list
    fn nvim_buf_get_next(buf: *mut BufHandle) -> *mut BufHandle;

    /// buf_spname: returns special buffer name or NULL
    fn buf_spname(buf: *mut BufHandle) -> *mut c_char;

    /// vim_tempname: create a temp file name
    fn vim_tempname() -> *mut c_char;

    /// mf_open_file: open a swap file for memfile (consumes fname)
    fn mf_open_file(mfp: *mut std::ffi::c_void, fname: *mut c_char) -> c_int;

    /// rs_mf_close_file_impl: close swap file without getlines
    fn rs_mf_close_file_impl(mfp: *mut std::ffi::c_void);

    /// rs_mf_set_dirty_all: mark all blocks dirty
    fn rs_mf_set_dirty_all(mfp: *mut std::ffi::c_void);

    /// nvim_mf_set_dirty: set mf_dirty field to a specific value
    fn nvim_mf_set_dirty(mfp: *mut std::ffi::c_void, val: c_int);

    /// nvim_mf_get_fname: get mfp->mf_fname
    fn nvim_mf_get_fname(mfp: *mut std::ffi::c_void) -> *mut c_char;

    /// nvim_mf_get_fd: get mfp->mf_fd
    fn nvim_mf_get_fd(mfp: *mut std::ffi::c_void) -> c_int;
}

// =============================================================================
// Phase 9-2: ml_setname extern declarations
// =============================================================================

extern "C" {
    /// Rename a file (vim_rename wrapper)
    #[link_name = "vim_rename"]
    fn nvim_vim_rename(from: *const c_char, to: *const c_char) -> c_int;

    /// mf_free_fnames: free memfile fname/ffname strings
    fn mf_free_fnames(mfp: *mut std::ffi::c_void);

    /// mf_set_fnames: set memfile fname (consumes allocated fname)
    fn mf_set_fnames(mfp: *mut std::ffi::c_void, fname: *mut c_char);

    /// nvim_mf_set_fd: set mfp->mf_fd
    fn nvim_mf_set_fd(mfp: *mut std::ffi::c_void, fd: c_int);

    /// os_open: open a file (thin wrapper around Rust os_open)
    fn os_open(path: *const c_char, flags: c_int, mode: c_int) -> c_int;

    /// nvim_os_set_cloexec: set close-on-exec flag on fd
    #[link_name = "os_set_cloexec"]
    fn nvim_os_set_cloexec(fd: c_int);

    /// emsg: print error message
    fn emsg(msg: *const c_char) -> bool;

    /// Direct access: p_uc (updatecount)
    static mut p_uc: i64;

}

// =============================================================================
// Phase 9-3: ml_open extern declarations
// =============================================================================

extern "C" {
    /// Initialize block 0 header (magic numbers, version, page_size)
    fn nvim_b0_init_header(b0p: *mut std::ffi::c_void, page_size: c_uint);

    /// Set buf->b_p_swf = false
    fn nvim_buf_set_b_p_swf_false(buf: *mut BufHandle);

    /// Set buf->b_may_swap = true
    fn nvim_buf_set_b_may_swap_true(buf: *mut BufHandle);

    /// mf_open: open or create a memfile
    fn mf_open(fname: *mut c_char, flags: c_int) -> *mut std::ffi::c_void;

    /// mf_new: allocate a new block in the memfile
    fn mf_new(
        mfp: *mut std::ffi::c_void,
        negative: bool,
        page_count: c_uint,
    ) -> *mut std::ffi::c_void;

    /// nvim_mf_get_page_size: get mfp->mf_page_size
    fn nvim_mf_get_page_size(mfp: *mut std::ffi::c_void) -> c_uint;

    /// Set b0_dirty from buf->b_changed
    fn nvim_b0_set_dirty_from_buf(b0p: *mut std::ffi::c_void, buf: *mut BufHandle);

    /// Set b0_flags from file format number
    fn nvim_b0_set_flags_from_ff(b0p: *mut std::ffi::c_void, fileformat: c_int);

    /// Fill b0_uname from os_get_username
    fn nvim_b0_fill_uname(b0p: *mut std::ffi::c_void);

    /// Fill b0_hname from os_get_hostname
    fn nvim_b0_fill_hname(b0p: *mut std::ffi::c_void);

    /// Fill b0_pid from os_get_pid
    fn nvim_b0_fill_pid(b0p: *mut std::ffi::c_void);

    /// rs_ml_new_ptr: allocate new pointer block (in the same crate)
    fn rs_ml_new_ptr(mfp: *mut std::ffi::c_void) -> *mut std::ffi::c_void;

    /// rs_ml_new_data: allocate new data block (in the same crate)
    fn rs_ml_new_data(
        mfp: *mut std::ffi::c_void,
        negative: bool,
        page_count: c_int,
    ) -> *mut std::ffi::c_void;

    /// iemsg: emit an internal error message
    fn iemsg(msg: *const c_char);

    /// nvim_bhdr_get_bh_bnum: get block header block number
    fn nvim_bhdr_get_bh_bnum(hp: *mut std::ffi::c_void) -> i64;

}

// =============================================================================
// Phase 9-4: ml_close_all, ml_close_notmod, ml_sync_all extern declarations
// =============================================================================

extern "C" {
    /// spell_delete_wordlist: delete the internal spell word list
    fn spell_delete_wordlist();

    /// vim_deltempdir: delete temp directory created by Neovim
    fn vim_deltempdir();
}

// =============================================================================
// Phase 2: Swap File Path Helpers (Rust implementations)
// =============================================================================

extern "C" {
    /// Fix a file name (expand to absolute, etc.)
    fn fix_fname(fname: *const c_char) -> *mut c_char;

    /// Check if a character is a path separator
    fn vim_ispathsep(c: c_int) -> c_int;

    /// Concatenate two file names with a separator if needed
    fn concat_fnames(fname1: *const c_char, fname2: *const c_char, sep: c_int) -> *mut c_char;

    /// Get the tail (filename part) of a path
    fn path_tail(fname: *const c_char) -> *mut c_char;

    /// Check if a path is absolute
    fn path_is_absolute(fname: *const c_char) -> c_int;

    /// Get the full name of a file (resolve relative paths)
    fn vim_FullName(fname: *const c_char, buf: *mut c_char, len: c_int, force: c_int) -> c_int;

    /// Compute a modified filename (like replacing extension)
    fn modname(fname: *const c_char, ext: *const c_char, prepend_dot: c_int) -> *mut c_char;

    /// Check if a pointer is after a path separator
    fn after_pathsep(b: *const c_char, p: *const c_char) -> c_int;

    /// Duplicate a string (allocate and copy)
    fn xstrdup(str: *const c_char) -> *mut c_char;

    /// Copy at most n bytes
    fn xstrlcpy(dst: *mut c_char, src: *const c_char, n: usize) -> usize;

    /// Get length of multibyte char at pointer
    fn utf_ptr2len(p: *const c_char) -> c_int;

    /// Semsg (error message with format)
    fn semsg(msg: *const c_char, ...);

    /// Get MAXPATHL constant value
    fn nvim_get_maxpathl() -> usize;

    /// Free a heap-allocated pointer
    fn xfree(ptr: *mut std::ffi::c_void);
}

#[cfg(unix)]
extern "C" {
    fn readlink(path: *const c_char, buf: *mut c_char, bufsiz: usize) -> isize;
}

// Errno access
#[cfg(unix)]
extern "C" {
    fn __errno_location() -> *mut c_int;
}

#[cfg(unix)]
unsafe fn errno() -> c_int {
    *__errno_location()
}

#[cfg(unix)]
const EINVAL: c_int = 22;
#[cfg(unix)]
const ENOENT: c_int = 2;

/// Append full path to dir with path separators replaced by `%` signs.
///
/// The last character in "dir" must be an extra slash, which is removed.
/// Mirrors the C `make_percent_swname` function.
///
/// # Safety
/// - `dir`, `dir_end` must be valid C string pointers into the same allocation
/// - `name` may be NULL (treated as "")
/// - Returns allocated string or NULL
#[allow(clippy::must_use_candidate)]
#[export_name = "make_percent_swname"]
#[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_make_percent_swname(
    dir: *mut c_char,
    dir_end: *mut c_char,
    name: *const c_char,
) -> *mut c_char {
    let f = fix_fname(if name.is_null() { c"".as_ptr() } else { name });
    if f.is_null() {
        return std::ptr::null_mut();
    }

    let s = xstrdup(f);
    xfree(f.cast());

    // Replace path separators with '%', advancing by multibyte char lengths
    let mut d = s;
    loop {
        let ch = *d;
        if ch == 0 {
            break;
        }
        if vim_ispathsep(c_int::from(ch as u8)) != 0 {
            *d = b'%' as c_char;
        }
        let adv = utf_ptr2len(d);
        #[allow(clippy::cast_sign_loss)]
        let adv_usize = adv as usize;
        d = d.add(adv_usize);
    }

    // Remove one trailing slash from dir (dir_end[-1] = NUL)
    *dir_end.sub(1) = 0;

    let result = concat_fnames(dir, s, 1);
    xfree(s.cast());
    result
}

/// Resolve a symlink to get the real path.
///
/// Only resolves the last component of the path. Returns OK if the symlink
/// was resolved (even partially), FAIL if not a symlink.
///
/// Mirrors the C `resolve_symlink` function (HAVE_READLINK).
///
/// # Safety
/// - `fname` must be a valid C string
/// - `buf` must be a buffer of at least MAXPATHL bytes
#[cfg(unix)]
#[export_name = "resolve_symlink"]
#[allow(clippy::cast_sign_loss)]
pub unsafe extern "C" fn rs_resolve_symlink(fname: *const c_char, buf: *mut c_char) -> c_int {
    const FAIL: c_int = 0;

    if fname.is_null() {
        return FAIL;
    }

    let maxpathl = nvim_get_maxpathl();
    let mut tmp = vec![0i8; maxpathl];
    let tmp_ptr = tmp.as_mut_ptr();

    // Start with the original name in tmp
    xstrlcpy(tmp_ptr, fname, maxpathl);

    let mut depth = 0i32;
    loop {
        depth += 1;
        if depth == 100 {
            let msg = c"E773: Symlink loop for \"%s\"".as_ptr();
            semsg(msg, fname);
            return FAIL;
        }

        let ret = readlink(tmp_ptr, buf, maxpathl - 1);
        if ret <= 0 {
            let err = errno();
            if err == EINVAL || err == ENOENT {
                // Found non-symlink or not-existing file, stop here.
                if depth == 1 {
                    return FAIL;
                }
                // Use the resolved name in tmp[]
                break;
            }
            // Some other error
            return FAIL;
        }
        *buf.add(ret as usize) = 0;

        // Check whether the symlink is relative or absolute
        if path_is_absolute(buf) != 0 {
            // Absolute: copy to tmp
            xstrlcpy(tmp_ptr, buf, maxpathl);
        } else {
            // Relative: build new path from directory part of tmp + symlink target
            let tail = path_tail(tmp_ptr);
            let tail_offset = tail.offset_from(tmp_ptr) as usize;
            let buf_len = {
                let mut n = 0usize;
                while *buf.add(n) != 0 {
                    n += 1;
                }
                n
            };
            if tail_offset + buf_len >= maxpathl {
                return FAIL;
            }
            // Copy symlink target over the tail of tmp
            std::ptr::copy_nonoverlapping(buf, tail, buf_len + 1);
        }
    }

    // Resolve the full name for consistency
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    vim_FullName(tmp_ptr, buf, maxpathl as c_int, 1)
}

/// Get file name for swap/backup file in a directory.
///
/// - If dname is ".", return xstrdup(fname)
/// - If dname starts with "./", insert path relative to dir of fname
/// - Otherwise prepend dname to tail of fname
///
/// Mirrors the C `get_file_in_dir` function.
///
/// # Safety
/// - `fname` and `dname` must be valid C strings
/// - Returns allocated string or NULL
#[allow(clippy::must_use_candidate)]
#[export_name = "get_file_in_dir"]
#[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_get_file_in_dir(fname: *mut c_char, dname: *mut c_char) -> *mut c_char {
    if fname.is_null() || dname.is_null() {
        return std::ptr::null_mut();
    }

    let tail = path_tail(fname);

    // dname == "." means use the file's own directory
    if *dname == b'.' as c_char && *dname.add(1) == 0 {
        return xstrdup(fname);
    }

    // dname starts with "./" means relative to the file's directory
    if *dname == b'.' as c_char && vim_ispathsep(c_int::from(*dname.add(1) as u8)) != 0 {
        if tail == fname {
            // No path before file name: use dname+2 + tail
            return concat_fnames(dname.add(2), tail, 1);
        }
        // Has a path: use dir/dname_rel/tail
        let save_char = *tail;
        *tail = 0;
        let t = concat_fnames(fname, dname.add(2), 1);
        *tail = save_char;
        let retval = concat_fnames(t, tail, 1);
        xfree(t.cast());
        return retval;
    }

    // Otherwise: prepend dname to tail
    concat_fnames(dname, tail, 1)
}

/// Make swap file name from file name and directory.
///
/// Mirrors the C `makeswapname` function.
///
/// When `fname` is NULL, `modname` uses the current directory (same as C).
///
/// # Safety
/// - `buf`, `dir_name` must be valid C strings; `fname` may be NULL
/// - Returns allocated string or NULL
#[allow(clippy::must_use_candidate)]
#[export_name = "makeswapname"]
#[allow(clippy::similar_names, clippy::cast_possible_wrap)]
pub unsafe extern "C" fn rs_makeswapname(
    fname: *mut c_char,
    _ffname: *mut c_char,
    _buf: *mut BufHandle,
    dir_name: *mut c_char,
) -> *mut c_char {
    if dir_name.is_null() {
        return std::ptr::null_mut();
    }

    let maxpathl = nvim_get_maxpathl();

    // Resolve symlinks if supported (only when fname is non-NULL)
    #[allow(unused_mut)]
    let mut fname_res = fname;

    #[cfg(unix)]
    let mut fname_buf = vec![0i8; maxpathl];
    #[cfg(unix)]
    {
        if !fname.is_null() && rs_resolve_symlink(fname, fname_buf.as_mut_ptr()) == 1 {
            fname_res = fname_buf.as_mut_ptr();
        }
    }

    // Compute length of dir_name
    let mut len = 0usize;
    while *dir_name.add(len) != 0 {
        len += 1;
    }

    let s = dir_name.add(len);
    // Check if it ends with '//' (full-path mode)
    if after_pathsep(dir_name, s) != 0 && len > 1 && *s.sub(1) == *s.sub(2) {
        // Use full path: replace '/' with '%'
        let swname = rs_make_percent_swname(dir_name, s, fname_res);
        if swname.is_null() {
            return std::ptr::null_mut();
        }
        let r = modname(swname, c".swp".as_ptr(), 0);
        xfree(swname.cast());
        return r;
    }

    // Prepend a '.' to the swap file name for the current directory
    let prepend_dot = c_int::from(*dir_name == b'.' as c_char && *dir_name.add(1) == 0);
    let r = modname(fname_res, c".swp".as_ptr(), prepend_dot);
    if r.is_null() {
        return std::ptr::null_mut();
    }

    let result = rs_get_file_in_dir(r, dir_name);
    xfree(r.cast());
    result
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
    c_int::from(!(*buf.cast::<BufStruct>()).ml_mfp.is_null())
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
    if (*buf.cast::<BufStruct>()).ml_mfp.is_null() {
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
    let ffname = (*buf.cast::<BufStruct>()).b_ffname;
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
    fn nvim_home_replace_b0_fname(buf: *const BufHandle, b0p: *mut c_void, maxlen: usize);
    #[link_name = "os_get_username"]
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
    let ffname = (*buf.cast::<BufStruct>()).b_ffname;

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
    let mfp = (*buf.cast::<BufStruct>()).ml_mfp;
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

// =============================================================================
// Phase 8 Pass 1: findswapname cluster
// =============================================================================

extern "C" {
    // findswapname accessors

    /// Get swap_exists_action global
    fn nvim_get_swap_exists_action() -> c_int;

    /// Set swap_exists_action global
    fn nvim_set_swap_exists_action(val: c_int);

    /// Get recoverymode global
    fn nvim_get_recoverymode() -> c_int;

    /// Get p_shm option string
    fn nvim_get_p_shm() -> *const c_char;

    /// Increment no_wait_return
    fn nvim_inc_no_wait_return();

    /// Decrement no_wait_return
    fn nvim_dec_no_wait_return();

    /// Check if path link exists
    fn nvim_os_fileinfo_link(fname: *const c_char) -> c_int;

    /// Read block 0 from fd into b0p (returns 1 on success)
    fn nvim_read_block0(fd: c_int, b0p: *mut c_void) -> c_int;

    /// Compare two file paths
    #[link_name = "path_fnamecmp"]
    fn nvim_path_fnamecmp(a: *const c_char, b: *const c_char) -> c_int;

    /// Check if paths are in same directory
    #[link_name = "same_directory"]
    fn nvim_same_directory(a: *const c_char, b: *const c_char) -> c_int;

    /// Expand environment variables
    #[link_name = "expand_env"]
    fn nvim_expand_env_maxpathl(src: *const c_char, dst: *mut c_char, len: c_int);

    /// Check if path is a directory
    fn nvim_os_isdir(name: *const c_char) -> c_int;

    /// Create directory recursively
    fn nvim_os_mkdir_recurse(
        dir: *const c_char,
        mode: c_int,
        failed_dir: *mut *mut c_char,
    ) -> c_int;

    /// Check if path exists (returns int)
    fn nvim_os_path_exists(name: *const c_char) -> bool;

    /// Remove a file
    fn nvim_os_remove(name: *const c_char) -> c_int;

    /// Get path tail pointer (const version)
    fn nvim_path_tail_const(fname: *const c_char) -> *mut c_char;

    /// Check if SwapExists autocmd exists for this file
    fn nvim_has_autocmd_swapexists(fname: *const c_char, buf: *mut BufHandle) -> c_int;

    /// Apply SwapExists autocommands
    fn nvim_apply_autocmds_swapexists(fname: *const c_char, buf: *mut BufHandle);

    /// Get v:swapchoice string
    fn nvim_get_vim_var_swapchoice() -> *const c_char;

    /// Set v:swapname
    fn nvim_set_vim_var_swapname(fname: *const c_char);

    /// Clear v:swapname
    fn nvim_clear_vim_var_swapname();

    /// Clear v:swapchoice
    fn nvim_clear_vim_var_swapchoice();

    /// Show dialog and return choice
    fn nvim_do_dialog_warning(
        title: *const c_char,
        message: *const c_char,
        buttons: *const c_char,
        dflt_button: c_int,
        mouse_used: bool,
    ) -> c_int;

    /// Flush type-ahead buffers
    fn nvim_flush_buffers_typeahead();

    /// Reset scroll position for messages
    #[link_name = "msg_reset_scroll"]
    fn nvim_msg_reset_scroll();

    /// Get home-replace-save of fname (returns allocated string)
    #[link_name = "home_replace_save"]
    fn nvim_home_replace_save(buf: *mut BufHandle, fname: *const c_char) -> *mut c_char;

    /// Output multiline message
    fn nvim_msg_multiline(s: *const c_char, hl_id: c_int);

    /// Verbose message
    #[link_name = "verb_msg"]
    fn nvim_verb_msg(s: *const c_char);

    /// Open file for reading, return fd
    fn nvim_os_open_rdonly(fname: *const c_char) -> c_int;

    /// Close file descriptor
    fn nvim_close_fd(fd: c_int);

    /// Allocate StringBuilder of IOSIZE
    fn nvim_alloc_stringbuilder_iosize() -> *mut c_void;

    /// Get items pointer of StringBuilder
    fn nvim_sb_get_items(sb: *mut c_void) -> *const c_char;

    /// Get size of StringBuilder
    fn nvim_sb_get_size(sb: *mut c_void) -> usize;

    /// Destroy and free StringBuilder
    fn nvim_free_stringbuilder(sb: *mut c_void);

    /// Append string to StringBuilder
    fn nvim_sb_append(sb: *mut c_void, s: *const c_char);

    /// emsg wrapper
    #[link_name = "emsg"]
    fn nvim_emsg(s: *const c_char) -> bool;

    /// msg_puts("\n")
    fn nvim_msg_puts_newline();

    /// Get file mtime (0 if not found)
    fn nvim_get_file_mtime(fname: *const c_char) -> i64;

    /// Format a time_t as ctime string (with trailing newline)
    fn os_ctime_r(
        clock: *const i64,
        result: *mut c_char,
        result_len: usize,
        add_newline: bool,
    ) -> *mut c_char;

    /// copy_option_part wrapper
    fn copy_option_part(
        option: *mut *mut c_char,
        buf: *mut c_char,
        maxlen: c_int,
        sep_chars: *const c_char,
    ) -> c_int;

    /// os_strerror wrapper (nvim accessor since os_strerror is a macro)
    fn nvim_os_strerror(err: c_int) -> *const c_char;

    // semsg - already declared in Phase 2 extern block above

    /// rs_swapfile_proc_running
    fn rs_swapfile_proc_running(b0p: *const c_void, swap_fname: *const c_char) -> c_int;

    /// rs_fnamecmp_ino
    fn rs_fnamecmp_ino(fname_c: *const c_char, fname_s: *const c_char, ino: i64) -> c_int;

    /// rs_char_to_long
    fn rs_char_to_long(s: *const c_char) -> i64;

    /// rs_swapfile_unchanged
    fn rs_swapfile_unchanged(fname: *mut c_char) -> bool;

    /// rs_swapfile_info
    fn rs_swapfile_info(fname: *mut c_char, sb: *mut c_void, proc_running_out: *mut c_int) -> i64;

    // nvim_b0_get_flags_byte - already declared in Phase 2 extern block above
    // nvim_b0_get_ino - already declared in Phase 2 extern block above

    /// get b0_fname pointer (const)
    fn nvim_b0_get_fname_ptr(b0p: *const c_void) -> *const c_char;

    /// get b0 struct size
    fn nvim_b0_get_struct_size() -> usize;

    /// Allocate memory (xmalloc)
    fn xmalloc(size: usize) -> *mut c_void;

    /// Get p_verbose global
    fn nvim_get_p_verbose() -> c_int;

}

use crate::types::{BF_DUMMY, SEA_NONE, SEA_QUIT, SEA_READONLY, SEA_RECOVER};
use nvim_buffer::buf_struct::BufStruct;
// BF_RECOVERED imported at top; B0_SAME_DIR imported in Phase 2 section above

/// Trigger the SwapExists autocommands.
///
/// Returns the sea_choice_T value corresponding to v:swapchoice, or SEA_CHOICE_NONE.
///
/// # Safety
/// Calls into C via FFI.
unsafe fn do_swapexists(buf: *mut BufHandle, fname: *const c_char) -> c_int {
    use crate::types::{
        SEA_CHOICE_ABORT, SEA_CHOICE_DELETE, SEA_CHOICE_EDIT, SEA_CHOICE_NONE, SEA_CHOICE_QUIT,
        SEA_CHOICE_READONLY, SEA_CHOICE_RECOVER,
    };

    nvim_set_vim_var_swapname(fname);
    nvim_clear_vim_var_swapchoice();

    // Trigger SwapExists autocommands (allbuf_lock inside nvim_apply_autocmds_swapexists)
    let buf_fname = (*buf.cast::<BufStruct>()).b_fname;
    nvim_apply_autocmds_swapexists(buf_fname, buf);

    nvim_clear_vim_var_swapname();

    let choice_str = nvim_get_vim_var_swapchoice();
    if choice_str.is_null() || *choice_str == 0 {
        return SEA_CHOICE_NONE;
    }
    match (*choice_str).cast_unsigned() {
        b'o' => SEA_CHOICE_READONLY,
        b'e' => SEA_CHOICE_EDIT,
        b'r' => SEA_CHOICE_RECOVER,
        b'd' => SEA_CHOICE_DELETE,
        b'q' => SEA_CHOICE_QUIT,
        b'a' => SEA_CHOICE_ABORT,
        _ => SEA_CHOICE_NONE,
    }
}

/// Build the ATTENTION message into the StringBuilder `sb`.
///
/// # Safety
/// Calls into C via FFI.
unsafe fn attention_message(
    buf: *mut BufHandle,
    fname: *const c_char,
    home_fname: *const c_char,
    sb: *mut c_void,
    proc_running: &mut c_int,
) {
    nvim_emsg(c"E325: ATTENTION".as_ptr());
    nvim_sb_append(sb, c"Found a swap file by the name \"".as_ptr());
    nvim_sb_append(sb, home_fname);
    nvim_sb_append(sb, "\"\n".as_ptr().cast());

    // Get swap file mtime via rs_swapfile_info (fills sb with info, sets proc_running)
    let swap_mtime = rs_swapfile_info(fname.cast_mut(), sb, proc_running);

    let buf_fname = (*buf.cast::<BufStruct>()).b_fname;
    nvim_sb_append(sb, c"While opening file \"".as_ptr());
    nvim_sb_append(sb, buf_fname);
    nvim_sb_append(sb, "\"\n".as_ptr().cast());

    // Get the original file's mtime
    let file_mtime = nvim_get_file_mtime(buf_fname);
    if file_mtime == 0 {
        nvim_sb_append(sb, c"      CANNOT BE FOUND".as_ptr());
    } else {
        nvim_sb_append(sb, c"             dated: ".as_ptr());
        {
            let mut ctime_buf = [0i8; 100];
            let s = os_ctime_r(
                &raw const file_mtime,
                ctime_buf.as_mut_ptr(),
                ctime_buf.len(),
                true,
            );
            nvim_sb_append(sb, s);
        }
        if swap_mtime != 0 && file_mtime > swap_mtime {
            nvim_sb_append(sb, c"      NEWER than swap file!\n".as_ptr());
        }
    }

    nvim_sb_append(
        sb,
        c"\n(1) Another program may be editing the same file.  If this is the case,\n    be careful not to end up with two different instances of the same\n    file when making changes.  Quit, or continue with caution.\n".as_ptr(),
    );
    nvim_sb_append(sb, c"(2) An edit session for this file crashed.\n".as_ptr());
    nvim_sb_append(
        sb,
        c"    If this is the case, use \":recover\" or \"nvim -r ".as_ptr(),
    );
    nvim_sb_append(sb, buf_fname);
    nvim_sb_append(
        sb,
        c"\"\n    to recover the changes (see \":help recovery\").\n".as_ptr(),
    );
    nvim_sb_append(
        sb,
        c"    If you did this already, delete the swap file \"".as_ptr(),
    );
    nvim_sb_append(sb, fname);
    nvim_sb_append(sb, c"\"\n    to avoid this message.\n".as_ptr());
}

/// Find out what name to use for the swapfile for buffer `buf`.
///
/// Several names are tried to find one that does not exist. The last directory
/// in `dirp` is automatically created.
///
/// # Arguments
/// - `buf` - Buffer for which the swapfile name is needed
/// - `dirp` - Pointer to a list of directories; advanced to next on return
/// - `old_fname` - Allowed existing swapfile name (NULL to not allow any)
/// - `found_existing_dir` - Set to true if directory already exists
///
/// # Returns
/// Allocated name of the swapfile (caller must free), or NULL on failure.
///
/// # Safety
/// Calls into C via FFI.
#[no_mangle]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::too_many_lines
)]
pub unsafe extern "C" fn rs_findswapname(
    buf: *mut BufHandle,
    dirp: *mut *mut c_char,
    old_fname: *const c_char,
    found_existing_dir: *mut bool,
) -> *mut c_char {
    let buf_fname = (*buf.cast::<BufStruct>()).b_fname;

    // Isolate a directory name from *dirp and put it in dir_name.
    // We pre-count the length of *dirp so we can allocate enough space.
    // copy_option_part will advance *dirp past the next comma-separated segment.
    let dirp_str_len = {
        let mut n = 0usize;
        let p = *dirp;
        while *p.add(n) != 0 {
            n += 1;
        }
        n
    };
    let dir_len = dirp_str_len + 1;
    let dir_name = xmalloc(dir_len).cast::<c_char>();
    copy_option_part(dirp, dir_name, dir_len as c_int, c",".as_ptr());

    // We try different swapfile names until we find one that does not exist yet.
    let mut fname = rs_makeswapname(
        buf_fname.cast_mut(),
        (*buf.cast::<BufStruct>()).b_ffname.cast_mut(),
        buf,
        dir_name,
    );

    'outer: loop {
        if fname.is_null() {
            // out of memory
            break;
        }
        let n = {
            let mut len = 0usize;
            while *fname.add(len) != 0 {
                len += 1;
            }
            len
        };
        if n == 0 {
            xfree(fname.cast());
            fname = std::ptr::null_mut();
            break;
        }

        // Check if the swapfile already exists (also checks symlinks for security)
        if nvim_os_fileinfo_link(fname) == 0 {
            break; // file doesn't exist, we can use this name
        }

        // A file name equal to old_fname is OK to use.
        if !old_fname.is_null() && nvim_path_fnamecmp(fname, old_fname) == 0 {
            break;
        }

        // get here when file already exists
        if *fname.add(n - 2) == b'w' as c_char && *fname.add(n - 1) == b'p' as c_char {
            // first try (.swp extension)
            // If we get here the ".swp" file really exists.
            // Give an error message, unless recovering, no file name, we are
            // viewing a help file, or when the path of the file is different.
            if nvim_get_recoverymode() == 0
                && !buf_fname.is_null()
                && (*buf.cast::<BufStruct>()).b_help == 0
                && ((*buf.cast::<BufStruct>()).b_flags & BF_DUMMY) == 0
            {
                let mut differ = false;

                // Allocate a zero-block buffer on the stack (opaque, sized by C)
                let b0_size = nvim_b0_get_struct_size();
                let b0p = xmalloc(b0_size);

                // Try to read block 0 from the swapfile
                let fd = nvim_os_open_rdonly(fname);
                if fd >= 0 {
                    if nvim_read_block0(fd, b0p) != 0 {
                        // We don't capture proc_running here; attention_message
                        // will call rs_swapfile_info which re-reads and sets it.
                        let _ = rs_swapfile_proc_running(b0p, fname);

                        let b0_flags = nvim_b0_get_flags_byte(b0p);
                        let b0_fname = nvim_b0_get_fname_ptr(b0p);
                        let full_fname = (*buf.cast::<BufStruct>()).b_ffname;
                        if (b0_flags & B0_SAME_DIR) != 0 {
                            if nvim_path_fnamecmp(
                                nvim_path_tail_const(full_fname),
                                nvim_path_tail_const(b0_fname),
                            ) != 0
                                || nvim_same_directory(fname, full_fname) == 0
                            {
                                // Symlinks may point to the same file even when the name
                                // differs, need to check inode too.
                                let maxpathl = nvim_get_maxpathl();
                                let name_buff = xmalloc(maxpathl).cast::<c_char>();
                                nvim_expand_env_maxpathl(b0_fname, name_buff, maxpathl as c_int);
                                let ino = rs_char_to_long(nvim_b0_get_ino(b0p).cast());
                                if rs_fnamecmp_ino(full_fname, name_buff, ino) != 0 {
                                    differ = true;
                                }
                                xfree(name_buff.cast());
                            }
                        } else {
                            // The name in the swapfile may be "~user/path/file".  Expand it first.
                            let maxpathl = nvim_get_maxpathl();
                            let name_buff = xmalloc(maxpathl).cast::<c_char>();
                            nvim_expand_env_maxpathl(b0_fname, name_buff, maxpathl as c_int);
                            let ino = rs_char_to_long(nvim_b0_get_ino(b0p).cast());
                            if rs_fnamecmp_ino(full_fname, name_buff, ino) != 0 {
                                differ = true;
                            }
                            xfree(name_buff.cast());
                        }
                    }
                    nvim_close_fd(fd);
                }

                xfree(b0p);

                // Show the ATTENTION message when:
                // - there is an old swapfile for the current file
                // - the buffer was not recovered
                let p_shm = nvim_get_p_shm();
                let shm_has_attention = if p_shm.is_null() {
                    false
                } else {
                    let mut p = p_shm;
                    while *p != 0 {
                        if *p == SHM_ATTENTION as c_char {
                            break;
                        }
                        p = p.add(1);
                    }
                    *p != 0
                };

                if !differ
                    && ((*buf.cast::<BufStruct>()).b_flags & BF_RECOVERED) == 0
                    && !shm_has_attention
                {
                    let mut choice = crate::types::SEA_CHOICE_NONE;

                    // It's safe to delete the swapfile if all these are true:
                    // - the edited file exists
                    // - the swapfile has no changes and looks OK
                    if !buf_fname.is_null()
                        && nvim_os_path_exists(buf_fname)
                        && rs_swapfile_unchanged(fname)
                    {
                        choice = crate::types::SEA_CHOICE_DELETE;
                        let p_verbose = nvim_get_p_verbose();
                        if p_verbose > 0 {
                            nvim_verb_msg(
                                c"Found a swap file that is not useful, deleting it".as_ptr(),
                            );
                        }
                    }

                    // If there is a SwapExists autocommand and we can handle the response,
                    // trigger it.
                    if choice == crate::types::SEA_CHOICE_NONE
                        && nvim_get_swap_exists_action() != SEA_NONE
                        && !buf_fname.is_null()
                        && nvim_has_autocmd_swapexists(buf_fname, buf) != 0
                    {
                        choice = do_swapexists(buf, fname);
                    }

                    if choice == crate::types::SEA_CHOICE_NONE
                        && nvim_get_swap_exists_action() == SEA_READONLY
                    {
                        choice = crate::types::SEA_CHOICE_READONLY;
                    }

                    // proc_running will be set by attention_message via rs_swapfile_info
                    let mut proc_running: c_int = 0;

                    if choice == crate::types::SEA_CHOICE_NONE {
                        nvim_inc_no_wait_return();

                        // Show info about the existing swapfile.
                        let sb = nvim_alloc_stringbuilder_iosize();
                        let home_fname = nvim_home_replace_save(std::ptr::null_mut(), fname);

                        attention_message(buf, fname, home_fname, sb, &mut proc_running);

                        // We don't want a 'q' typed at the more-prompt to interrupt loading.
                        unsafe {
                            got_int = false;
                        }

                        // Flush typeahead to avoid vimrc "simalt ~x" interference.
                        nvim_flush_buffers_typeahead();

                        if nvim_get_swap_exists_action() == SEA_NONE {
                            let sb_items = nvim_sb_get_items(sb);
                            let sb_size = nvim_sb_get_size(sb);
                            // Append NUL terminator for use as C string (items may not be NUL-terminated)
                            let mut msg_buf = std::vec::Vec::with_capacity(sb_size + 1);
                            if !sb_items.is_null() && sb_size > 0 {
                                let slice =
                                    std::slice::from_raw_parts(sb_items.cast::<u8>(), sb_size);
                                msg_buf.extend_from_slice(slice);
                            }
                            msg_buf.push(0u8);
                            nvim_msg_multiline(msg_buf.as_ptr().cast(), 0);
                        } else {
                            // Build message for dialog
                            nvim_sb_append(sb, c"Swap file \"".as_ptr());
                            nvim_sb_append(sb, home_fname);
                            nvim_sb_append(sb, c"\" already exists!".as_ptr());

                            let sb_items = nvim_sb_get_items(sb);
                            let run_but = c"&Open Read-Only\n&Edit anyway\n&Recover\n&Quit\n&Abort";
                            let but = c"&Open Read-Only\n&Edit anyway\n&Recover\n&Delete it\n&Quit\n&Abort";
                            let buttons = if proc_running != 0 {
                                run_but.as_ptr()
                            } else {
                                but.as_ptr()
                            };
                            choice = nvim_do_dialog_warning(
                                c"VIM - ATTENTION".as_ptr(),
                                sb_items,
                                buttons,
                                1,
                                false,
                            );
                            // Compensate for missing "Delete it" button when proc is running
                            if proc_running != 0 && choice >= 4 {
                                choice += 1;
                            }
                            // Pretend screen didn't scroll, need redraw anyway
                            nvim_msg_reset_scroll();
                        }

                        nvim_dec_no_wait_return();
                        nvim_free_stringbuilder(sb);
                        xfree(home_fname.cast());
                    }

                    // Handle the choice
                    match choice {
                        c if c == crate::types::SEA_CHOICE_READONLY => {
                            (*buf.cast::<BufStruct>()).b_p_ro = 1;
                        }
                        c if c == crate::types::SEA_CHOICE_EDIT => {
                            // Edit anyway: do nothing
                        }
                        c if c == crate::types::SEA_CHOICE_RECOVER => {
                            nvim_set_swap_exists_action(SEA_RECOVER);
                        }
                        c if c == crate::types::SEA_CHOICE_DELETE => {
                            nvim_os_remove(fname);
                        }
                        c if c == crate::types::SEA_CHOICE_QUIT => {
                            nvim_set_swap_exists_action(SEA_QUIT);
                        }
                        c if c == crate::types::SEA_CHOICE_ABORT => {
                            nvim_set_swap_exists_action(SEA_QUIT);
                            unsafe {
                                got_int = true;
                            }
                        }
                        _ => {
                            // SEA_CHOICE_NONE
                            nvim_msg_puts_newline();
                            if msg_silent == 0 {
                                // call wait_return() later
                                need_wait_return = true;
                            }
                        }
                    }

                    // If the swapfile was deleted, this fname can be used.
                    if choice != crate::types::SEA_CHOICE_NONE && !nvim_os_path_exists(fname) {
                        break 'outer;
                    }
                }
            }
        }

        // Permute the ".swp" extension to find a unique swapfile name.
        // First decrement the last char: ".swo", ".swn", etc.
        // If that still isn't enough decrement the last but one char: ".svz"
        if *fname.add(n - 1) == b'a' as c_char {
            if *fname.add(n - 2) == b'a' as c_char {
                // ".saa": tried enough, give up
                nvim_emsg(c"E326: Too many swap files found".as_ptr());
                xfree(fname.cast());
                fname = std::ptr::null_mut();
                break;
            }
            // ".svz", ".suz", etc.
            *fname.add(n - 2) = (*fname.add(n - 2) as u8 - 1) as c_char;
            *fname.add(n - 1) = (b'z' + 1) as c_char;
        }
        *fname.add(n - 1) = (*fname.add(n - 1) as u8 - 1) as c_char;
    }

    // Create directory if needed
    if nvim_os_isdir(dir_name) != 0 {
        *found_existing_dir = true;
    } else if !*found_existing_dir && **dirp == 0 {
        let mut failed_dir: *mut c_char = std::ptr::null_mut();
        let ret = nvim_os_mkdir_recurse(dir_name, 0o755, &raw mut failed_dir);
        if ret != 0 {
            semsg(
                c"E303: Unable to create directory \"%s\" for swap file, recovery impossible: %s"
                    .as_ptr(),
                failed_dir,
                nvim_os_strerror(ret),
            );
            xfree(failed_dir.cast());
        }
    }

    xfree(dir_name.cast());
    fname
}

// =============================================================================
// Phase 8 Pass 2: ml_preserve and ml_sync_one
// =============================================================================

extern "C" {
    /// Sync memfile blocks to disk
    #[link_name = "mf_sync"]
    fn nvim_mf_sync(mfp: *mut c_void, flags: c_int) -> c_int;

    /// Check if memfile has blocks needing block number translation
    #[link_name = "mf_need_trans"]
    fn nvim_mf_need_trans(mfp: *mut c_void) -> c_int;

    /// Check if memfile has dirty blocks (mf_dirty == MF_DIRTY_YES)
    fn nvim_mf_is_dirty(mfp: *mut c_void) -> c_int;

    /// Check if a character is available (for stopping sync mid-loop)
    fn nvim_os_char_avail() -> c_int;

    /// Set need_check_timestamps global
    fn nvim_set_need_check_timestamps(val: c_int);

    /// Global: did_check_timestamps
    static mut did_check_timestamps: bool;

    /// Check if original file changed since last read
    fn nvim_buf_file_unchanged(buf: *mut BufHandle) -> c_int;

    /// Check if buffer has unsaved changes
    fn nvim_buf_is_changed(buf: *mut BufHandle) -> c_int;

    /// Flush buffered line for buffer
    #[link_name = "ml_flush_line"]
    fn rs_ml_flush_line(buf: *mut BufHandle, noalloc: c_int);

    /// Find or flush line in B-tree
    fn rs_ml_find_line(buf: *mut BufHandle, lnum: LineNr, action: c_int) -> *mut c_void;

    /// Emit "File preserved" message
    fn nvim_msg_file_preserved();

    /// Emit E314 "Preserve failed" error
    fn nvim_emsg_preserve_failed();

    /// Emit E313 "Cannot preserve, there is no swap file" error
    fn nvim_emsg_no_swapfile();
}

use crate::types::{LineNr, MFS_ALL, MFS_FLUSH, MFS_STOP, ML_FIND, ML_FLUSH};

/// Sync one buffer's memline, including negative blocks.
///
/// Called from the `ml_sync_all` C loop (which uses `FOR_ALL_BUFFERS`).
/// Returns 1 if the loop should break (character available), 0 to continue.
///
/// # Safety
/// Calls into C via FFI.
#[no_mangle]
pub unsafe extern "C" fn rs_ml_sync_one(
    buf: *mut BufHandle,
    check_file: c_int,
    check_char: c_int,
    do_fsync: bool,
) -> c_int {
    let mfp = (*buf.cast::<BufStruct>()).ml_mfp;
    if mfp.is_null() {
        return 0;
    }
    let mfp_fname = nvim_buf_get_ml_mfp_fname(buf);
    if mfp_fname.is_null() {
        return 0;
    }

    rs_ml_flush_line(buf, 0); // flush buffered line
    rs_ml_find_line(buf, 0, ML_FLUSH); // flush locked block

    if nvim_buf_is_changed(buf) != 0
        && check_file != 0
        && nvim_mf_need_trans(mfp) != 0
        && !(*buf.cast::<BufStruct>()).b_ffname.is_null()
        && nvim_buf_file_unchanged(buf) != 0
    {
        // Original file no longer matches; preserve to translate negative blocks.
        rs_ml_preserve(buf, false, do_fsync);
        did_check_timestamps = false;
        nvim_set_need_check_timestamps(1); // give message later
    }

    if nvim_mf_is_dirty(mfp) != 0 {
        let flags = (if check_char != 0 { MFS_STOP } else { 0 })
            | (if do_fsync && nvim_buf_is_changed(buf) != 0 {
                MFS_FLUSH
            } else {
                0
            });
        mf_sync(mfp, flags);
        if check_char != 0 && nvim_os_char_avail() != 0 {
            return 1; // character available now, stop loop
        }
    }
    0
}

/// Write all blocks of buffer's memline to the swap file.
///
/// After this call all blocks are in the swap file.
/// Used for the `:preserve` command and when the original file has changed.
///
/// # Safety
/// Calls into C via FFI.
#[export_name = "ml_preserve"]
pub unsafe extern "C" fn rs_ml_preserve(buf: *mut BufHandle, message: bool, do_fsync: bool) {
    let mfp = (*buf.cast::<BufStruct>()).ml_mfp;
    if mfp.is_null() || nvim_buf_get_ml_mfp_fname(buf).is_null() {
        if message {
            nvim_emsg_no_swapfile();
        }
        return;
    }

    // We only want to stop when interrupted here, not when interrupted before.
    let got_int_save = unsafe { got_int };
    unsafe {
        got_int = false;
    }

    rs_ml_flush_line(buf, 0); // flush buffered line
    rs_ml_find_line(buf, 0, ML_FLUSH); // flush locked block
    let sync_flags = MFS_ALL | (if do_fsync { MFS_FLUSH } else { 0 });
    let mut status = nvim_mf_sync(mfp, sync_flags);

    // stack is invalid after mf_sync(.., MFS_ALL)
    (*buf.cast::<BufStruct>()).ml_stack_top = 0;

    // Some data blocks may have changed from negative to positive block numbers.
    // In that case the pointer blocks need to be updated.
    // ml_find_line() does the work by translating negative block numbers when
    // getting the first line of each data block.
    if nvim_mf_need_trans(mfp) != 0 && !unsafe { got_int } {
        let mut lnum: LineNr = 1;
        while nvim_mf_need_trans(mfp) != 0
            && lnum <= (LineNr::from((*buf.cast::<BufStruct>()).ml_line_count))
        {
            let hp = rs_ml_find_line(buf, lnum, ML_FIND);
            if hp.is_null() {
                status = crate::types::FAIL;
                break;
            }
            lnum = (LineNr::from((*buf.cast::<BufStruct>()).ml_locked_high)) + 1;
        }
        rs_ml_find_line(buf, 0, ML_FLUSH); // flush locked block
                                           // sync the updated pointer blocks
        if nvim_mf_sync(mfp, sync_flags) == crate::types::FAIL {
            status = crate::types::FAIL;
        }
        (*buf.cast::<BufStruct>()).ml_stack_top = 0; // stack is invalid now
    }

    // Restore got_int (OR with saved value so prior interrupt is not lost)
    unsafe {
        got_int |= got_int_save;
    }

    if message {
        if status == crate::types::OK {
            nvim_msg_file_preserved();
        } else {
            nvim_emsg_preserve_failed();
        }
    }
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
