//! Swap file and memory block management for Neovim
//!
//! This module provides types and utilities for managing memory-mapped files
//! used for swap files and buffer content storage.

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

use std::ffi::c_int;

// =============================================================================
// Block Number Type
// =============================================================================

/// Block number type (matches blocknr_T in C)
pub type BlockNr = i64;

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

/// Get BH_DIRTY flag value
#[no_mangle]
pub extern "C" fn rs_bh_dirty() -> u32 {
    BlockFlags::DIRTY.bits()
}

/// Get BH_LOCKED flag value
#[no_mangle]
pub extern "C" fn rs_bh_locked() -> u32 {
    BlockFlags::LOCKED.bits()
}

/// Check if block is dirty
#[no_mangle]
pub extern "C" fn rs_block_is_dirty(flags: u32) -> bool {
    BlockFlags::from_bits_truncate(flags).contains(BlockFlags::DIRTY)
}

/// Check if block is locked
#[no_mangle]
pub extern "C" fn rs_block_is_locked(flags: u32) -> bool {
    BlockFlags::from_bits_truncate(flags).contains(BlockFlags::LOCKED)
}

/// Set dirty flag
#[no_mangle]
pub extern "C" fn rs_block_set_dirty(flags: u32) -> u32 {
    (BlockFlags::from_bits_truncate(flags) | BlockFlags::DIRTY).bits()
}

/// Clear dirty flag
#[no_mangle]
pub extern "C" fn rs_block_clear_dirty(flags: u32) -> u32 {
    (BlockFlags::from_bits_truncate(flags) & !BlockFlags::DIRTY).bits()
}

/// Set locked flag
#[no_mangle]
pub extern "C" fn rs_block_set_locked(flags: u32) -> u32 {
    (BlockFlags::from_bits_truncate(flags) | BlockFlags::LOCKED).bits()
}

/// Clear locked flag
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
    /// No dirty blocks
    No = 0,
    /// There are dirty blocks
    Yes = 1,
    /// There are dirty blocks, do not sync yet
    YesNoSync = 2,
}

impl MemfileDirty {
    /// Create from integer value
    pub fn from_int(value: c_int) -> Option<Self> {
        match value {
            0 => Some(Self::No),
            1 => Some(Self::Yes),
            2 => Some(Self::YesNoSync),
            _ => None,
        }
    }

    /// Check if there are any dirty blocks
    pub fn has_dirty(self) -> bool {
        self != Self::No
    }

    /// Check if sync should be performed
    pub fn should_sync(self) -> bool {
        self == Self::Yes
    }
}

/// Get MF_DIRTY_NO value
#[no_mangle]
pub extern "C" fn rs_mf_dirty_no() -> c_int {
    MemfileDirty::No as c_int
}

/// Get MF_DIRTY_YES value
#[no_mangle]
pub extern "C" fn rs_mf_dirty_yes() -> c_int {
    MemfileDirty::Yes as c_int
}

/// Get MF_DIRTY_YES_NOSYNC value
#[no_mangle]
pub extern "C" fn rs_mf_dirty_yes_nosync() -> c_int {
    MemfileDirty::YesNoSync as c_int
}

/// Check if memfile has dirty blocks
#[no_mangle]
pub extern "C" fn rs_mf_has_dirty(state: c_int) -> bool {
    MemfileDirty::from_int(state).is_some_and(|s| s.has_dirty())
}

/// Check if memfile should sync
#[no_mangle]
pub extern "C" fn rs_mf_should_sync(state: c_int) -> bool {
    MemfileDirty::from_int(state).is_some_and(|s| s.should_sync())
}

// =============================================================================
// Page Size Constants
// =============================================================================

/// Default memfile page size
pub const MEMFILE_PAGE_SIZE: usize = 4096;

/// Get default page size
#[no_mangle]
pub extern "C" fn rs_memfile_page_size() -> usize {
    MEMFILE_PAGE_SIZE
}

/// Check if page count is valid (positive)
#[no_mangle]
pub extern "C" fn rs_page_count_valid(count: u32) -> bool {
    count > 0
}

/// Calculate total size from page count
#[no_mangle]
pub extern "C" fn rs_pages_to_bytes(page_count: u32, page_size: usize) -> usize {
    page_count as usize * page_size
}

/// Calculate page count from size (rounding up)
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

/// State for block number translation (negative to positive)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BlockTranslation {
    /// Original negative block number
    pub from_bnum: BlockNr,
    /// New positive block number
    pub to_bnum: BlockNr,
}

/// Create a block translation entry
#[no_mangle]
pub extern "C" fn rs_block_translation_new(from: BlockNr, to: BlockNr) -> BlockTranslation {
    BlockTranslation {
        from_bnum: from,
        to_bnum: to,
    }
}

/// Check if translation is valid
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

/// Swap file open flags
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwapOpenMode {
    /// Open existing swap file for recovery
    Recovery = 0,
    /// Create new swap file
    Create = 1,
}

/// Get recovery mode value
#[no_mangle]
pub extern "C" fn rs_swap_mode_recovery() -> c_int {
    SwapOpenMode::Recovery as c_int
}

/// Get create mode value
#[no_mangle]
pub extern "C" fn rs_swap_mode_create() -> c_int {
    SwapOpenMode::Create as c_int
}

/// Check if mode is for recovery
#[no_mangle]
pub extern "C" fn rs_swap_is_recovery(mode: c_int) -> bool {
    mode == SwapOpenMode::Recovery as c_int
}

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
}
