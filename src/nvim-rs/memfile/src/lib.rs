//! Swap file and memory block management for Neovim
//!
//! This module provides types and utilities for managing memory-mapped files
//! used for swap files and buffer content storage.
//!
//! A memfile consists of a sequence of blocks:
//! - Blocks numbered from 0 upwards have been assigned a place in the actual
//!   file. The block number is equal to the page number in the file.
//! - Blocks with negative numbers are currently in memory only. They can be
//!   assigned a place in the file when too much memory is being used. At that
//!   moment, they get a new, positive, number. A list is used for translation
//!   of negative to positive numbers.

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::manual_div_ceil)]
#![allow(clippy::borrow_as_ptr)]

use std::ffi::{c_char, c_int, c_uint, c_void, CStr};
use std::ptr;

// =============================================================================
// Opaque Handle Types
// =============================================================================

/// Opaque handle to memfile_T (C struct)
type MfHandle = *mut c_void;

/// Opaque handle to bhdr_T (C struct)
type BhHandle = *mut c_void;

/// Block number type (matches blocknr_T in C)
pub type BlockNr = i64;

/// off_T (matches off_t / loff_t in C)
type OffT = i64;

// =============================================================================
// FFI Constants (verified against C headers with _Static_assert)
// =============================================================================

const BH_DIRTY: c_uint = 1;
const BH_LOCKED: c_uint = 2;

const MF_DIRTY_NO: c_int = 0;
const MF_DIRTY_YES: c_int = 1;
const MF_DIRTY_YES_NOSYNC: c_int = 2;

const MFS_ALL: c_int = 1;
const MFS_STOP: c_int = 2;
const MFS_FLUSH: c_int = 4;
const MFS_ZERO: c_int = 8;

const MIN_SWAP_PAGE_SIZE: c_uint = 1048;
const MAX_SWAP_PAGE_SIZE: c_uint = 50000;

const OK: c_int = 1;
const FAIL: c_int = 0;

const MEMFILE_PAGE_SIZE_DEFAULT: c_uint = 4096;

// =============================================================================
// C Accessor Functions (extern "C")
// =============================================================================

extern "C" {
    // --- memfile_T field accessors ---
    fn nvim_mf_get_fname(mfp: MfHandle) -> *mut c_char;
    fn nvim_mf_set_fname(mfp: MfHandle, fname: *mut c_char);
    fn nvim_mf_get_ffname(mfp: MfHandle) -> *mut c_char;
    fn nvim_mf_set_ffname(mfp: MfHandle, ffname: *mut c_char);
    fn nvim_mf_get_fd(mfp: MfHandle) -> c_int;
    fn nvim_mf_set_fd(mfp: MfHandle, fd: c_int);
    fn nvim_mf_get_flags(mfp: MfHandle) -> c_int;
    fn nvim_mf_set_flags(mfp: MfHandle, flags: c_int);
    fn nvim_mf_get_reopen(mfp: MfHandle) -> bool;
    fn nvim_mf_set_reopen(mfp: MfHandle, reopen: bool);
    fn nvim_mf_get_free_first(mfp: MfHandle) -> BhHandle;
    fn nvim_mf_set_free_first(mfp: MfHandle, hp: BhHandle);
    fn nvim_mf_get_blocknr_max(mfp: MfHandle) -> BlockNr;
    fn nvim_mf_set_blocknr_max(mfp: MfHandle, val: BlockNr);
    fn nvim_mf_get_blocknr_min(mfp: MfHandle) -> BlockNr;
    fn nvim_mf_set_blocknr_min(mfp: MfHandle, val: BlockNr);
    fn nvim_mf_get_neg_count(mfp: MfHandle) -> BlockNr;
    fn nvim_mf_set_neg_count(mfp: MfHandle, val: BlockNr);
    fn nvim_mf_get_infile_count(mfp: MfHandle) -> BlockNr;
    fn nvim_mf_set_infile_count(mfp: MfHandle, val: BlockNr);
    fn nvim_mf_get_page_size(mfp: MfHandle) -> c_uint;
    fn nvim_mf_set_page_size(mfp: MfHandle, val: c_uint);
    fn nvim_mf_get_dirty(mfp: MfHandle) -> c_int;
    fn nvim_mf_set_dirty(mfp: MfHandle, val: c_int);

    // --- bhdr_T field accessors ---
    fn nvim_bh_get_bnum(hp: BhHandle) -> BlockNr;
    fn nvim_bh_set_bnum(hp: BhHandle, bnum: BlockNr);
    fn nvim_bh_get_data(hp: BhHandle) -> *mut c_void;
    fn nvim_bh_set_data(hp: BhHandle, data: *mut c_void);
    fn nvim_bh_get_page_count(hp: BhHandle) -> c_uint;
    fn nvim_bh_set_page_count(hp: BhHandle, count: c_uint);
    fn nvim_bh_get_flags(hp: BhHandle) -> c_uint;
    fn nvim_bh_set_flags(hp: BhHandle, flags: c_uint);

    // --- Allocation wrappers ---
    fn nvim_mf_alloc() -> MfHandle;
    fn nvim_mf_dealloc(mfp: MfHandle);
    fn nvim_bh_alloc() -> BhHandle;
    fn nvim_bh_dealloc(hp: BhHandle);

    // --- Memory allocation ---
    fn xmalloc(size: usize) -> *mut c_void;
    fn xfree(ptr: *mut c_void);

    // --- Map/PMap wrappers ---
    fn nvim_mf_hash_init(mfp: MfHandle);
    fn nvim_mf_hash_destroy(mfp: MfHandle);
    fn nvim_mf_hash_get(mfp: MfHandle, key: BlockNr) -> BhHandle;
    fn nvim_mf_hash_put(mfp: MfHandle, key: BlockNr, hp: BhHandle);
    fn nvim_mf_hash_del(mfp: MfHandle, key: BlockNr);
    fn nvim_mf_hash_size(mfp: MfHandle) -> c_int;
    fn nvim_mf_hash_value_at(mfp: MfHandle, index: c_int) -> BhHandle;
    fn nvim_mf_trans_init(mfp: MfHandle);
    fn nvim_mf_trans_destroy(mfp: MfHandle);
    fn nvim_mf_trans_put(mfp: MfHandle, old_bnum: BlockNr, new_bnum: BlockNr);
    fn nvim_mf_trans_ref(mfp: MfHandle, old_bnum: BlockNr) -> *mut BlockNr;
    fn nvim_mf_trans_del(mfp: MfHandle, old_bnum: BlockNr);

    // --- Global variable accessors ---
    fn nvim_mf_get_got_int() -> c_int;
    fn nvim_mf_set_got_int(val: c_int);
    fn nvim_mf_get_did_swapwrite_msg() -> c_int;
    fn nvim_mf_set_did_swapwrite_msg(val: c_int);

    // --- Message wrappers ---
    fn nvim_mf_emsg(msg: *const c_char);
    fn nvim_mf_iemsg(msg: *const c_char);
    fn nvim_mf_perror(msg: *const c_char);

    // --- FileInfo wrappers ---
    fn nvim_mf_fileinfo_fd(fd: c_int, blocksize_out: *mut u64) -> bool;
    fn nvim_mf_fileinfo_link_exists(fname: *const c_char) -> bool;

    // --- File I/O wrappers ---
    fn nvim_mf_os_open(fname: *const c_char, flags: c_int, mode: c_int) -> c_int;
    fn nvim_mf_os_remove(fname: *const c_char);
    fn nvim_mf_os_set_cloexec(fd: c_int);
    fn nvim_mf_os_fsync(fd: c_int) -> c_int;
    fn nvim_mf_os_char_avail() -> bool;
    fn nvim_mf_os_breakcheck();
    fn nvim_mf_vim_lseek(fd: c_int, offset: OffT, whence: c_int) -> OffT;
    fn nvim_mf_read_eintr(fd: c_int, buf: *mut c_void, size: c_uint) -> c_int;
    fn nvim_mf_write_eintr(fd: c_int, buf: *const c_void, size: c_uint) -> c_int;
    fn nvim_mf_close_fd(fd: c_int) -> c_int;

    // --- String/path wrappers ---
    fn nvim_mf_fullname_save(fname: *const c_char) -> *mut c_char;
    fn nvim_mf_xfree_clear_fname(mfp: MfHandle);
    fn nvim_mf_xfree_clear_ffname(mfp: MfHandle);

}

// SEEK_SET / SEEK_END constants from libc
const SEEK_SET: c_int = libc::SEEK_SET;
const SEEK_END: c_int = libc::SEEK_END;

// O_* flags from libc
const O_RDWR: c_int = libc::O_RDWR;
const O_CREAT: c_int = libc::O_CREAT;
const O_EXCL: c_int = libc::O_EXCL;
const O_TRUNC: c_int = libc::O_TRUNC;
const O_NOFOLLOW: c_int = libc::O_NOFOLLOW;

// File permission constants
const S_IREAD: c_int = libc::S_IRUSR as c_int;
const S_IWRITE: c_int = libc::S_IWUSR as c_int;

// =============================================================================
// Error messages (English text; C wrapper applies _() for localization)
// =============================================================================

const E293_BLOCK_NOT_LOCKED: &CStr = c"E293: Block was not locked";
const E294_SEEK_READ: &CStr = c"E294: Seek error in swap file read";
const E295_READ_ERROR: &CStr = c"E295: Read error in swap file";
const E296_SEEK_WRITE: &CStr = c"E296: Seek error in swap file write";
const E297_WRITE_ERROR: &CStr = c"E297: Write error in swap file";
const E300_SWAP_EXISTS: &CStr = c"E300: Swap file already exists (symlink attack?)";

// =============================================================================
// Block Number Utility Functions
// =============================================================================

/// Check if block number is positive (assigned to file)
#[no_mangle]
pub extern "C" fn rs_blocknr_is_positive(bnum: BlockNr) -> bool {
    bnum >= 0
}

/// Check if block number is negative (memory only)
#[no_mangle]
pub extern "C" fn rs_blocknr_is_negative(bnum: BlockNr) -> bool {
    bnum < 0
}

/// Check if block number is valid (non-zero)
#[no_mangle]
pub extern "C" fn rs_blocknr_is_valid(bnum: BlockNr) -> bool {
    bnum != 0
}

// =============================================================================
// Block Header Flags
// =============================================================================

bitflags::bitflags! {
    /// Block header flags
    #[repr(C)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct BlockFlags: u32 {
        /// Block has been modified
        const DIRTY = 1;
        /// Block is currently locked (in use)
        const LOCKED = 2;
    }
}

#[no_mangle]
pub extern "C" fn rs_bh_dirty() -> u32 {
    BlockFlags::DIRTY.bits()
}

#[no_mangle]
pub extern "C" fn rs_bh_locked() -> u32 {
    BlockFlags::LOCKED.bits()
}

#[no_mangle]
pub extern "C" fn rs_block_is_dirty(flags: u32) -> bool {
    BlockFlags::from_bits_truncate(flags).contains(BlockFlags::DIRTY)
}

#[no_mangle]
pub extern "C" fn rs_block_is_locked(flags: u32) -> bool {
    BlockFlags::from_bits_truncate(flags).contains(BlockFlags::LOCKED)
}

#[no_mangle]
pub extern "C" fn rs_block_set_dirty(flags: u32) -> u32 {
    (BlockFlags::from_bits_truncate(flags) | BlockFlags::DIRTY).bits()
}

#[no_mangle]
pub extern "C" fn rs_block_clear_dirty(flags: u32) -> u32 {
    (BlockFlags::from_bits_truncate(flags) & !BlockFlags::DIRTY).bits()
}

#[no_mangle]
pub extern "C" fn rs_block_set_locked(flags: u32) -> u32 {
    (BlockFlags::from_bits_truncate(flags) | BlockFlags::LOCKED).bits()
}

#[no_mangle]
pub extern "C" fn rs_block_clear_locked(flags: u32) -> u32 {
    (BlockFlags::from_bits_truncate(flags) & !BlockFlags::LOCKED).bits()
}

// =============================================================================
// Memfile Dirty State
// =============================================================================

/// Memfile dirty state (matches mfdirty_T in C)
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemfileDirty {
    No = 0,
    Yes = 1,
    YesNoSync = 2,
}

impl MemfileDirty {
    pub fn from_int(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::No),
            1 => Some(Self::Yes),
            2 => Some(Self::YesNoSync),
            _ => None,
        }
    }

    pub fn has_dirty(self) -> bool {
        self != Self::No
    }

    pub fn should_sync(self) -> bool {
        self == Self::Yes
    }
}

#[no_mangle]
pub extern "C" fn rs_mf_dirty_no() -> c_int {
    MemfileDirty::No as c_int
}

#[no_mangle]
pub extern "C" fn rs_mf_dirty_yes() -> c_int {
    MemfileDirty::Yes as c_int
}

#[no_mangle]
pub extern "C" fn rs_mf_dirty_yes_nosync() -> c_int {
    MemfileDirty::YesNoSync as c_int
}

#[no_mangle]
pub extern "C" fn rs_mf_has_dirty(state: c_int) -> bool {
    MemfileDirty::from_int(state).is_some_and(|s| s.has_dirty())
}

#[no_mangle]
pub extern "C" fn rs_mf_should_sync(state: c_int) -> bool {
    MemfileDirty::from_int(state).is_some_and(|s| s.should_sync())
}

// =============================================================================
// Page Size Constants
// =============================================================================

pub const MEMFILE_PAGE_SIZE: usize = 4096;

#[no_mangle]
pub extern "C" fn rs_memfile_page_size() -> usize {
    MEMFILE_PAGE_SIZE
}

#[no_mangle]
pub extern "C" fn rs_page_count_valid(count: u32) -> bool {
    count > 0
}

#[no_mangle]
pub extern "C" fn rs_pages_to_bytes(page_count: u32, page_size: usize) -> usize {
    page_count as usize * page_size
}

#[no_mangle]
pub extern "C" fn rs_bytes_to_pages(bytes: usize, page_size: usize) -> u32 {
    if page_size == 0 {
        return 0;
    }
    ((bytes + page_size - 1) / page_size) as u32
}

// =============================================================================
// Block Translation
// =============================================================================

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BlockTranslation {
    pub from_bnum: BlockNr,
    pub to_bnum: BlockNr,
}

#[no_mangle]
pub extern "C" fn rs_block_translation_new(from: BlockNr, to: BlockNr) -> BlockTranslation {
    BlockTranslation {
        from_bnum: from,
        to_bnum: to,
    }
}

#[no_mangle]
pub unsafe extern "C" fn rs_block_translation_valid(trans: *const BlockTranslation) -> bool {
    if trans.is_null() {
        return false;
    }
    let trans = &*trans;
    trans.from_bnum < 0 && trans.to_bnum >= 0
}

// =============================================================================
// Swap File Operations
// =============================================================================

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwapOpenMode {
    Recovery = 0,
    Create = 1,
}

#[no_mangle]
pub extern "C" fn rs_swap_mode_recovery() -> c_int {
    SwapOpenMode::Recovery as c_int
}

#[no_mangle]
pub extern "C" fn rs_swap_mode_create() -> c_int {
    SwapOpenMode::Create as c_int
}

#[no_mangle]
pub extern "C" fn rs_swap_is_recovery(mode: c_int) -> bool {
    mode == SwapOpenMode::Recovery as c_int
}

// =============================================================================
// Phase 2: Internal Helper Functions
// =============================================================================

/// Allocate a block header and a block of memory for it.
unsafe fn mf_alloc_bhdr(mfp: MfHandle, page_count: c_uint) -> BhHandle {
    let hp = nvim_bh_alloc();
    let size = nvim_mf_get_page_size(mfp) as usize * page_count as usize;
    let data = xmalloc(size);
    nvim_bh_set_data(hp, data);
    nvim_bh_set_page_count(hp, page_count);
    hp
}

/// Free a block header and its block memory.
unsafe fn mf_free_bhdr(hp: BhHandle) {
    xfree(nvim_bh_get_data(hp));
    nvim_bh_dealloc(hp);
}

/// Insert a block in the free list.
/// Note: bh_data is reused as a next-pointer in the free list.
unsafe fn mf_ins_free(mfp: MfHandle, hp: BhHandle) {
    nvim_bh_set_data(hp, nvim_mf_get_free_first(mfp));
    nvim_mf_set_free_first(mfp, hp);
}

/// Remove the first block in the free list and return it.
/// Caller must check that free_first is not NULL.
unsafe fn mf_rem_free(mfp: MfHandle) -> BhHandle {
    let hp = nvim_mf_get_free_first(mfp);
    nvim_mf_set_free_first(mfp, nvim_bh_get_data(hp) as BhHandle);
    hp
}

/// Read a block from disk.
unsafe fn mf_read(mfp: MfHandle, hp: BhHandle) -> c_int {
    let fd = nvim_mf_get_fd(mfp);
    if fd < 0 {
        return FAIL;
    }

    let page_size = nvim_mf_get_page_size(mfp);
    let bnum = nvim_bh_get_bnum(hp);
    let offset = (page_size as OffT) * bnum;
    if nvim_mf_vim_lseek(fd, offset, SEEK_SET) != offset {
        nvim_mf_perror(E294_SEEK_READ.as_ptr());
        return FAIL;
    }

    let page_count = nvim_bh_get_page_count(hp);
    let size = page_size * page_count;
    let data = nvim_bh_get_data(hp);
    if nvim_mf_read_eintr(fd, data, size) != size as c_int {
        nvim_mf_perror(E295_READ_ERROR.as_ptr());
        return FAIL;
    }

    OK
}

/// Write a block to disk. Most complex internal function: handles gap-filling,
/// retry on network disconnect, and reopen logic.
unsafe fn mf_write(mfp: MfHandle, hp: BhHandle) -> c_int {
    let fd = nvim_mf_get_fd(mfp);
    let reopen = nvim_mf_get_reopen(mfp);

    if fd < 0 && !reopen {
        return FAIL;
    }

    // Must assign file block number if negative
    let bnum = nvim_bh_get_bnum(hp);
    if bnum < 0 && mf_trans_add(mfp, hp) == FAIL {
        return FAIL;
    }

    let page_size = nvim_mf_get_page_size(mfp);

    // Write blocks to fill gaps in the file, then write the target block
    loop {
        let nr;
        let hp2;
        let bh_bnum = nvim_bh_get_bnum(hp);
        let infile_count = nvim_mf_get_infile_count(mfp);

        if bh_bnum > infile_count {
            // beyond end of file — fill gap
            nr = infile_count;
            hp2 = nvim_mf_hash_get(mfp, nr); // NULL caught below
        } else {
            nr = bh_bnum;
            hp2 = hp;
        }

        let offset = (page_size as OffT) * nr;
        let page_count = if hp2.is_null() {
            1
        } else {
            nvim_bh_get_page_count(hp2)
        };
        let size = page_size * page_count;

        let mut written = false;
        for attempt in 1..=2 {
            let cur_fd = nvim_mf_get_fd(mfp);
            if cur_fd >= 0 {
                if nvim_mf_vim_lseek(cur_fd, offset, SEEK_SET) != offset {
                    nvim_mf_perror(E296_SEEK_WRITE.as_ptr());
                    return FAIL;
                }
                let data = if hp2.is_null() {
                    nvim_bh_get_data(hp)
                } else {
                    nvim_bh_get_data(hp2)
                };
                if nvim_mf_write_eintr(cur_fd, data, size) == size as c_int {
                    written = true;
                    break;
                }
            }

            if attempt == 1 {
                // Try to reopen the file (network drive disconnect recovery)
                let cur_fd2 = nvim_mf_get_fd(mfp);
                if cur_fd2 >= 0 {
                    nvim_mf_close_fd(cur_fd2);
                }
                let fname = nvim_mf_get_fname(mfp);
                let flags = nvim_mf_get_flags(mfp);
                let new_fd = nvim_mf_os_open(fname, flags, S_IREAD | S_IWRITE);
                nvim_mf_set_fd(mfp, new_fd);
                nvim_mf_set_reopen(mfp, new_fd < 0);
            }
            if attempt == 2 || nvim_mf_get_fd(mfp) < 0 {
                if nvim_mf_get_did_swapwrite_msg() == 0 {
                    nvim_mf_emsg(E297_WRITE_ERROR.as_ptr());
                }
                nvim_mf_set_did_swapwrite_msg(1);
                return FAIL;
            }
        }

        if written {
            nvim_mf_set_did_swapwrite_msg(0);
            if !hp2.is_null() {
                let flags = nvim_bh_get_flags(hp2);
                nvim_bh_set_flags(hp2, flags & !BH_DIRTY);
            }
            let infile = nvim_mf_get_infile_count(mfp);
            if nr + (page_count as BlockNr) > infile {
                nvim_mf_set_infile_count(mfp, nr + page_count as BlockNr);
            }
            if nr == nvim_bh_get_bnum(hp) {
                break;
            }
        }
    }
    OK
}

/// Make block number positive and add it to the translation list.
unsafe fn mf_trans_add(mfp: MfHandle, hp: BhHandle) -> c_int {
    let bnum = nvim_bh_get_bnum(hp);
    if bnum >= 0 {
        return OK;
    }

    let page_count = nvim_bh_get_page_count(hp);
    let freep = nvim_mf_get_free_first(mfp);

    let new_bnum;
    if !freep.is_null() && nvim_bh_get_page_count(freep) >= page_count {
        new_bnum = nvim_bh_get_bnum(freep);
        if nvim_bh_get_page_count(freep) > page_count {
            nvim_bh_set_bnum(freep, nvim_bh_get_bnum(freep) + page_count as BlockNr);
            nvim_bh_set_page_count(freep, nvim_bh_get_page_count(freep) - page_count);
        } else {
            let removed = mf_rem_free(mfp);
            xfree(removed);
        }
    } else {
        new_bnum = nvim_mf_get_blocknr_max(mfp);
        nvim_mf_set_blocknr_max(mfp, new_bnum + page_count as BlockNr);
    }

    let old_bnum = nvim_bh_get_bnum(hp);
    nvim_mf_hash_del(mfp, old_bnum);
    nvim_bh_set_bnum(hp, new_bnum);
    nvim_mf_hash_put(mfp, new_bnum, hp);

    // Record translation from old negative to new positive
    nvim_mf_trans_put(mfp, old_bnum, new_bnum);

    OK
}

/// Open memfile's swapfile.
/// "fname" must be in allocated memory, and is consumed (also when error).
unsafe fn mf_do_open(mfp: MfHandle, fname: *mut c_char, flags: c_int) -> bool {
    // Set names (this consumes fname)
    rs_mf_set_fnames(mfp, fname);

    // Security check: when creating, the file should not already exist as a symlink
    if (flags & O_CREAT) != 0 && nvim_mf_fileinfo_link_exists(nvim_mf_get_fname(mfp)) {
        nvim_mf_set_fd(mfp, -1);
        nvim_mf_emsg(E300_SWAP_EXISTS.as_ptr());
    } else {
        let open_flags = flags | O_NOFOLLOW;
        nvim_mf_set_flags(mfp, open_flags);
        let fd = nvim_mf_os_open(nvim_mf_get_fname(mfp), open_flags, S_IREAD | S_IWRITE);
        nvim_mf_set_fd(mfp, fd);
    }

    if nvim_mf_get_fd(mfp) < 0 {
        rs_mf_free_fnames(mfp);
        return false;
    }

    nvim_mf_os_set_cloexec(nvim_mf_get_fd(mfp));
    true
}

// =============================================================================
// Phase 3: Simple Public Functions (10 functions)
// =============================================================================

/// Free mf_fname and mf_ffname.
#[export_name = "mf_free_fnames"]
pub unsafe extern "C" fn rs_mf_free_fnames(mfp: MfHandle) {
    nvim_mf_xfree_clear_fname(mfp);
    nvim_mf_xfree_clear_ffname(mfp);
}

/// Set the simple file name and the full file name of memfile's swapfile.
#[export_name = "mf_set_fnames"]
pub unsafe extern "C" fn rs_mf_set_fnames(mfp: MfHandle, fname: *mut c_char) {
    nvim_mf_set_fname(mfp, fname);
    let full_name = nvim_mf_fullname_save(nvim_mf_get_fname(mfp));
    nvim_mf_set_ffname(mfp, full_name);
}

/// Make name of memfile's swapfile a full path. Used before doing a :cd.
#[export_name = "mf_fullname"]
pub unsafe extern "C" fn rs_mf_fullname(mfp: MfHandle) {
    if mfp.is_null() {
        return;
    }
    let short_name = nvim_mf_get_fname(mfp);
    let full_path = nvim_mf_get_ffname(mfp);
    if short_name.is_null() || full_path.is_null() {
        return;
    }

    xfree(short_name.cast());
    nvim_mf_set_fname(mfp, full_path);
    nvim_mf_set_ffname(mfp, ptr::null_mut());
}

/// Return true if there are any translations pending for memfile.
#[export_name = "mf_need_trans"]
pub unsafe extern "C" fn rs_mf_need_trans(mfp: MfHandle) -> bool {
    !nvim_mf_get_fname(mfp).is_null() && nvim_mf_get_neg_count(mfp) > 0
}

/// Set new size for a memfile. Used when block 0 of a swapfile has been read
/// and the size it indicates differs from what was guessed.
#[export_name = "mf_new_page_size"]
pub unsafe extern "C" fn rs_mf_new_page_size(mfp: MfHandle, new_size: c_uint) {
    nvim_mf_set_page_size(mfp, new_size);
}

/// Lookup translation from trans list and delete the entry.
/// Returns the positive new number when found, or old_nr when not found.
#[export_name = "mf_trans_del"]
pub unsafe extern "C" fn rs_mf_trans_del(mfp: MfHandle, old_nr: BlockNr) -> BlockNr {
    let num_ptr = nvim_mf_trans_ref(mfp, old_nr);
    if num_ptr.is_null() {
        return old_nr;
    }

    let neg_count = nvim_mf_get_neg_count(mfp);
    nvim_mf_set_neg_count(mfp, neg_count - 1);
    let new_bnum = *num_ptr;

    nvim_mf_trans_del(mfp, old_nr);

    new_bnum
}

/// Release the block *hp.
#[export_name = "mf_put"]
pub unsafe extern "C" fn rs_mf_put(mfp: MfHandle, hp: BhHandle, dirty: bool, infile: bool) {
    let mut flags = nvim_bh_get_flags(hp);

    if (flags & BH_LOCKED) == 0 {
        nvim_mf_iemsg(E293_BLOCK_NOT_LOCKED.as_ptr());
    }
    flags &= !BH_LOCKED;
    if dirty {
        flags |= BH_DIRTY;
        if nvim_mf_get_dirty(mfp) != MF_DIRTY_YES_NOSYNC {
            nvim_mf_set_dirty(mfp, MF_DIRTY_YES);
        }
    }
    nvim_bh_set_flags(hp, flags);
    if infile {
        mf_trans_add(mfp, hp);
    }
}

/// Signal block as no longer used (may put it in the free list).
#[export_name = "mf_free"]
pub unsafe extern "C" fn rs_mf_free(mfp: MfHandle, hp: BhHandle) {
    xfree(nvim_bh_get_data(hp));
    let bnum = nvim_bh_get_bnum(hp);
    nvim_mf_hash_del(mfp, bnum);
    if bnum < 0 {
        xfree(hp);
        let neg = nvim_mf_get_neg_count(mfp);
        nvim_mf_set_neg_count(mfp, neg - 1);
    } else {
        mf_ins_free(mfp, hp);
    }
}

/// Open a file for an existing memfile.
/// Used when updatecount set from 0 to some value.
#[export_name = "mf_open_file"]
pub unsafe extern "C" fn rs_mf_open_file(mfp: MfHandle, fname: *mut c_char) -> c_int {
    if mf_do_open(mfp, fname, O_RDWR | O_CREAT | O_EXCL) {
        nvim_mf_set_dirty(mfp, MF_DIRTY_YES);
        return OK;
    }
    FAIL
}

/// Set dirty flag for all blocks in memory file with a positive block number.
#[no_mangle]
pub unsafe extern "C" fn rs_mf_set_dirty_all(mfp: MfHandle) {
    let size = nvim_mf_hash_size(mfp);
    for i in 0..size {
        let hp = nvim_mf_hash_value_at(mfp, i);
        if !hp.is_null() && nvim_bh_get_bnum(hp) > 0 {
            let flags = nvim_bh_get_flags(hp);
            nvim_bh_set_flags(hp, flags | BH_DIRTY);
        }
    }
    nvim_mf_set_dirty(mfp, MF_DIRTY_YES);
}

// =============================================================================
// Phase 4: Complex Public Functions (5 functions)
// =============================================================================

/// Open a new or existing memory block file.
#[export_name = "mf_open"]
pub unsafe extern "C" fn rs_mf_open(fname: *mut c_char, flags: c_int) -> MfHandle {
    let mfp = nvim_mf_alloc();

    if fname.is_null() {
        nvim_mf_set_fname(mfp, ptr::null_mut());
        nvim_mf_set_ffname(mfp, ptr::null_mut());
        nvim_mf_set_fd(mfp, -1);
    } else if !mf_do_open(mfp, fname, flags) {
        nvim_mf_dealloc(mfp);
        return ptr::null_mut();
    }

    nvim_mf_set_free_first(mfp, ptr::null_mut());
    nvim_mf_set_dirty(mfp, MF_DIRTY_NO);
    nvim_mf_hash_init(mfp);
    nvim_mf_trans_init(mfp);
    nvim_mf_set_page_size(mfp, MEMFILE_PAGE_SIZE_DEFAULT);

    // Try to set page size equal to device's block size
    let fd = nvim_mf_get_fd(mfp);
    if fd >= 0 {
        let mut blocksize: u64 = 0;
        if nvim_mf_fileinfo_fd(fd, &mut blocksize)
            && blocksize >= MIN_SWAP_PAGE_SIZE as u64
            && blocksize <= MAX_SWAP_PAGE_SIZE as u64
        {
            nvim_mf_set_page_size(mfp, blocksize as c_uint);
        }
    }

    let fd = nvim_mf_get_fd(mfp);
    let page_size = nvim_mf_get_page_size(mfp);

    if fd < 0 || (flags & (O_TRUNC | O_EXCL)) != 0 {
        nvim_mf_set_blocknr_max(mfp, 0);
    } else {
        let size = nvim_mf_vim_lseek(fd, 0, SEEK_END);
        if size <= 0 {
            nvim_mf_set_blocknr_max(mfp, 0);
        } else {
            let max = (size + page_size as OffT - 1) / page_size as OffT;
            nvim_mf_set_blocknr_max(mfp, max);
        }
    }

    nvim_mf_set_blocknr_min(mfp, -1);
    nvim_mf_set_neg_count(mfp, 0);
    nvim_mf_set_infile_count(mfp, nvim_mf_get_blocknr_max(mfp));

    mfp
}

/// Close a memory file and optionally delete the associated file.
#[export_name = "mf_close"]
pub unsafe extern "C" fn rs_mf_close(mfp: MfHandle, del_file: bool) {
    if mfp.is_null() {
        return;
    }

    let fd = nvim_mf_get_fd(mfp);
    if fd >= 0 && nvim_mf_close_fd(fd) < 0 {
        nvim_mf_emsg(c"E72: Close error on swap file".as_ptr());
    }

    if del_file {
        let fname = nvim_mf_get_fname(mfp);
        if !fname.is_null() {
            nvim_mf_os_remove(fname);
        }
    }

    // Free entries in used list
    let size = nvim_mf_hash_size(mfp);
    for i in 0..size {
        let hp = nvim_mf_hash_value_at(mfp, i);
        if !hp.is_null() {
            mf_free_bhdr(hp);
        }
    }

    // Free entries in free list
    while !nvim_mf_get_free_first(mfp).is_null() {
        let hp = mf_rem_free(mfp);
        xfree(hp);
    }

    nvim_mf_hash_destroy(mfp);
    nvim_mf_trans_destroy(mfp);
    rs_mf_free_fnames(mfp);
    nvim_mf_dealloc(mfp);
}

/// Create a new block in a memfile and lock it.
#[export_name = "mf_new"]
pub unsafe extern "C" fn rs_mf_new(mfp: MfHandle, negative: bool, page_count: c_uint) -> BhHandle {
    let hp;

    let freep = nvim_mf_get_free_first(mfp);
    if !negative && !freep.is_null() && nvim_bh_get_page_count(freep) >= page_count {
        if nvim_bh_get_page_count(freep) > page_count {
            // Take only the needed pages from the free block
            hp = mf_alloc_bhdr(mfp, page_count);
            nvim_bh_set_bnum(hp, nvim_bh_get_bnum(freep));
            nvim_bh_set_bnum(freep, nvim_bh_get_bnum(freep) + page_count as BlockNr);
            nvim_bh_set_page_count(freep, nvim_bh_get_page_count(freep) - page_count);
        } else {
            // Page count matches — take the bhdr from the free list, allocate data
            let page_sz = nvim_mf_get_page_size(mfp);
            let p = xmalloc(page_sz as usize * page_count as usize);
            hp = mf_rem_free(mfp);
            nvim_bh_set_data(hp, p);
        }
    } else {
        // Get a new number
        hp = mf_alloc_bhdr(mfp, page_count);
        if negative {
            let min = nvim_mf_get_blocknr_min(mfp);
            nvim_bh_set_bnum(hp, min);
            nvim_mf_set_blocknr_min(mfp, min - 1);
            let neg = nvim_mf_get_neg_count(mfp);
            nvim_mf_set_neg_count(mfp, neg + 1);
        } else {
            let max = nvim_mf_get_blocknr_max(mfp);
            nvim_bh_set_bnum(hp, max);
            nvim_mf_set_blocknr_max(mfp, max + page_count as BlockNr);
        }
    }

    nvim_bh_set_flags(hp, BH_LOCKED | BH_DIRTY);
    nvim_mf_set_dirty(mfp, MF_DIRTY_YES);
    nvim_bh_set_page_count(hp, page_count);
    nvim_mf_hash_put(mfp, nvim_bh_get_bnum(hp), hp);

    // Init the data to all zero, to avoid reading uninitialized data.
    let page_sz = nvim_mf_get_page_size(mfp);
    let data = nvim_bh_get_data(hp);
    ptr::write_bytes(data.cast::<u8>(), 0, page_sz as usize * page_count as usize);

    hp
}

/// Get an existing block and lock it.
/// Caller should first check a negative nr with mf_trans_del().
#[export_name = "mf_get"]
pub unsafe extern "C" fn rs_mf_get(mfp: MfHandle, nr: BlockNr, page_count: c_uint) -> BhHandle {
    let blocknr_max = nvim_mf_get_blocknr_max(mfp);
    let blocknr_min = nvim_mf_get_blocknr_min(mfp);

    if nr >= blocknr_max || nr <= blocknr_min {
        return ptr::null_mut();
    }

    let mut hp = nvim_mf_hash_get(mfp, nr);
    if hp.is_null() {
        let infile_count = nvim_mf_get_infile_count(mfp);
        if nr < 0 || nr >= infile_count {
            return ptr::null_mut();
        }

        if page_count > 0 {
            hp = mf_alloc_bhdr(mfp, page_count);
        }
        if hp.is_null() {
            return ptr::null_mut();
        }

        nvim_bh_set_bnum(hp, nr);
        nvim_bh_set_flags(hp, 0);
        nvim_bh_set_page_count(hp, page_count);
        if mf_read(mfp, hp) == FAIL {
            mf_free_bhdr(hp);
            return ptr::null_mut();
        }
    } else {
        nvim_mf_hash_del(mfp, nvim_bh_get_bnum(hp));
    }

    let flags = nvim_bh_get_flags(hp);
    nvim_bh_set_flags(hp, flags | BH_LOCKED);
    nvim_mf_hash_put(mfp, nvim_bh_get_bnum(hp), hp);

    hp
}

/// Sync memory file to disk.
#[export_name = "mf_sync"]
pub unsafe extern "C" fn rs_mf_sync(mfp: MfHandle, flags: c_int) -> c_int {
    let got_int_save = nvim_mf_get_got_int();

    if nvim_mf_get_fd(mfp) < 0 {
        nvim_mf_set_dirty(mfp, MF_DIRTY_NO);
        return FAIL;
    }

    // Only a CTRL-C while writing will break us here
    nvim_mf_set_got_int(0);

    let mut status = OK;
    let mut all_flushed = true;

    let size = nvim_mf_hash_size(mfp);
    for i in 0..size {
        let hp = nvim_mf_hash_value_at(mfp, i);
        if hp.is_null() {
            continue;
        }

        let bnum = nvim_bh_get_bnum(hp);
        let bh_flags = nvim_bh_get_flags(hp);
        let infile_count = nvim_mf_get_infile_count(mfp);

        if ((flags & MFS_ALL) != 0 || bnum >= 0)
            && (bh_flags & BH_DIRTY) != 0
            && (status == OK || (bnum >= 0 && bnum < infile_count))
        {
            if (flags & MFS_ZERO) != 0 && bnum != 0 {
                continue;
            }
            if mf_write(mfp, hp) == FAIL {
                if status == FAIL {
                    // double error: quit syncing
                    all_flushed = false;
                    break;
                }
                status = FAIL;
            }
            if (flags & MFS_STOP) != 0 {
                if nvim_mf_os_char_avail() {
                    all_flushed = false;
                    break;
                }
            } else {
                nvim_mf_os_breakcheck();
            }
            if nvim_mf_get_got_int() != 0 {
                all_flushed = false;
                break;
            }
        }
    }

    // If the whole list is flushed (or error), memfile is not dirty anymore
    if all_flushed || status == FAIL {
        nvim_mf_set_dirty(mfp, MF_DIRTY_NO);
    }

    if (flags & MFS_FLUSH) != 0 && nvim_mf_os_fsync(nvim_mf_get_fd(mfp)) != 0 {
        status = FAIL;
    }

    // Restore got_int (OR with saved value)
    let cur = nvim_mf_get_got_int();
    nvim_mf_set_got_int(cur | got_int_save);

    status
}

// =============================================================================
// Phase 5: Cross-Module Functions (split implementations)
// =============================================================================

/// Close the swap file for a memfile — Rust part.
/// The C stub handles buf_T/ml_get_buf loop; this handles close/remove/free.
#[no_mangle]
pub unsafe extern "C" fn rs_mf_close_file_impl(mfp: MfHandle) {
    let fd = nvim_mf_get_fd(mfp);
    if nvim_mf_close_fd(fd) < 0 {
        nvim_mf_emsg(c"E72: Close error on swap file".as_ptr());
    }
    nvim_mf_set_fd(mfp, -1);

    let fname = nvim_mf_get_fname(mfp);
    if !fname.is_null() {
        nvim_mf_os_remove(fname);
        rs_mf_free_fnames(mfp);
    }
}

/// Release blocks for a single memfile (inner loop of mf_release_all).
/// Returns true if any memory was released.
#[no_mangle]
pub unsafe extern "C" fn rs_mf_release_for_memfile(mfp: MfHandle) -> bool {
    let mut retval = false;
    let mut i = 0;
    while i < nvim_mf_hash_size(mfp) {
        let hp = nvim_mf_hash_value_at(mfp, i);
        if hp.is_null() {
            i += 1;
            continue;
        }
        let bh_flags = nvim_bh_get_flags(hp);
        if (bh_flags & BH_LOCKED) == 0 && ((bh_flags & BH_DIRTY) == 0 || mf_write(mfp, hp) != FAIL)
        {
            let bnum = nvim_bh_get_bnum(hp);
            nvim_mf_hash_del(mfp, bnum);
            mf_free_bhdr(hp);
            retval = true;
            // Rerun with same index — another item takes this slot
        } else {
            i += 1;
        }
    }
    retval
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blocknr() {
        assert!(rs_blocknr_is_positive(0));
        assert!(rs_blocknr_is_positive(100));
        assert!(!rs_blocknr_is_positive(-1));

        assert!(!rs_blocknr_is_negative(0));
        assert!(rs_blocknr_is_negative(-1));
        assert!(rs_blocknr_is_negative(-100));

        assert!(!rs_blocknr_is_valid(0));
        assert!(rs_blocknr_is_valid(1));
        assert!(rs_blocknr_is_valid(-1));
    }

    #[test]
    fn test_block_flags() {
        assert_eq!(rs_bh_dirty(), 1);
        assert_eq!(rs_bh_locked(), 2);

        let flags = 0u32;
        assert!(!rs_block_is_dirty(flags));
        assert!(!rs_block_is_locked(flags));

        let flags = rs_block_set_dirty(flags);
        assert!(rs_block_is_dirty(flags));
        assert!(!rs_block_is_locked(flags));

        let flags = rs_block_set_locked(flags);
        assert!(rs_block_is_dirty(flags));
        assert!(rs_block_is_locked(flags));

        let flags = rs_block_clear_dirty(flags);
        assert!(!rs_block_is_dirty(flags));
        assert!(rs_block_is_locked(flags));
    }

    #[test]
    fn test_memfile_dirty() {
        assert_eq!(rs_mf_dirty_no(), 0);
        assert_eq!(rs_mf_dirty_yes(), 1);
        assert_eq!(rs_mf_dirty_yes_nosync(), 2);

        assert!(!rs_mf_has_dirty(0));
        assert!(rs_mf_has_dirty(1));
        assert!(rs_mf_has_dirty(2));

        assert!(!rs_mf_should_sync(0));
        assert!(rs_mf_should_sync(1));
        assert!(!rs_mf_should_sync(2));
    }

    #[test]
    fn test_page_calculations() {
        assert_eq!(rs_memfile_page_size(), 4096);

        assert!(rs_page_count_valid(1));
        assert!(!rs_page_count_valid(0));

        assert_eq!(rs_pages_to_bytes(1, 4096), 4096);
        assert_eq!(rs_pages_to_bytes(2, 4096), 8192);

        assert_eq!(rs_bytes_to_pages(4096, 4096), 1);
        assert_eq!(rs_bytes_to_pages(4097, 4096), 2);
        assert_eq!(rs_bytes_to_pages(8192, 4096), 2);
    }

    #[test]
    fn test_block_translation() {
        unsafe {
            let trans = rs_block_translation_new(-5, 10);
            assert_eq!(trans.from_bnum, -5);
            assert_eq!(trans.to_bnum, 10);
            assert!(rs_block_translation_valid(&trans));

            let invalid = rs_block_translation_new(5, 10);
            assert!(!rs_block_translation_valid(&invalid));
        }
    }

    #[test]
    fn test_swap_mode() {
        assert_eq!(rs_swap_mode_recovery(), 0);
        assert_eq!(rs_swap_mode_create(), 1);

        assert!(rs_swap_is_recovery(0));
        assert!(!rs_swap_is_recovery(1));
    }

    #[test]
    fn test_constants() {
        // Verify our FFI constants match expected values
        assert_eq!(BH_DIRTY, 1);
        assert_eq!(BH_LOCKED, 2);
        assert_eq!(MF_DIRTY_NO, 0);
        assert_eq!(MF_DIRTY_YES, 1);
        assert_eq!(MF_DIRTY_YES_NOSYNC, 2);
        assert_eq!(MFS_ALL, 1);
        assert_eq!(MFS_STOP, 2);
        assert_eq!(MFS_FLUSH, 4);
        assert_eq!(MFS_ZERO, 8);
        assert_eq!(MIN_SWAP_PAGE_SIZE, 1048);
        assert_eq!(MAX_SWAP_PAGE_SIZE, 50000);
        assert_eq!(OK, 1);
        assert_eq!(FAIL, 0);
        assert_eq!(MEMFILE_PAGE_SIZE_DEFAULT, 4096);
    }
}
