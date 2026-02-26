//! Buffer memory management (memline) for Neovim
//!
//! This crate provides Rust implementations for the memline system, which manages
//! buffer text in a B-tree structure backed by a swap file for crash recovery.
//!
//! # Architecture
//!
//! The memline uses a tree structure with a branch factor of ~128:
//! - **Pointer blocks**: Internal nodes containing pointers to child blocks
//! - **Data blocks**: Leaf nodes containing actual line text
//! - **Block 0**: Special block with swap file metadata for recovery
//!
//! Lines are stored at leaf nodes (data blocks). The tree structure allows
//! efficient insertion and deletion without moving large amounts of memory.
//!
//! # Modules
//!
//! - [`types`]: Core data structures (B-tree nodes, handles), constants (flags,
//!   action codes, block IDs), and size calculations
//! - [`access`]: Line access functions - get line text and length, caching,
//!   data block index manipulation
//! - [`modify`]: Line modification - append, delete, replace operations with
//!   block splitting/merging support
//! - [`navigate`]: Cursor navigation, byte offset calculations, position
//!   validation, and B-tree traversal helpers
//! - [`swap`]: Swap file management - open/close, sync, block 0 field access,
//!   dirty state tracking
//! - [`recovery`]: Recovery from swap files - attention dialog handling,
//!   byte order detection, cross-platform recovery support
//!
//! # FFI Functions
//!
//! This crate exports 200+ functions for C interop, organized by category:
//!
//! - **Constant accessors** (`rs_ml_get_*`): Access memline constants
//! - **Line access** (`rs_ml_get_len`, `rs_ml_get_pos`, etc.): Read line content
//! - **Line modification** (`rs_ml_append`, `rs_ml_delete`, `rs_ml_replace`): Edit buffer
//! - **Navigation** (`rs_inc`, `rs_dec`, `rs_goto_byte`): Move cursor position
//! - **Swap file** (`rs_ml_open`, `rs_ml_sync_all`, `rs_b0_*`): Manage swap files
//! - **Recovery** (`rs_ml_recover`, `rs_sea_choice_*`): Handle crash recovery
//!
//! # Note
//!
//! This crate uses the opaque handle pattern for FFI safety. All C struct
//! access goes through accessor functions declared as `extern "C"`.

#![warn(clippy::all, clippy::pedantic, clippy::nursery)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
#![allow(unsafe_code)]

pub mod access;
pub mod chunk;
pub mod modify;
pub mod navigate;
pub mod recovery;
pub mod swap;
pub mod types;

// Re-export all public items for FFI
pub use access::*;
pub use chunk::*;
pub use modify::*;
pub use navigate::*;
pub use recovery::*;
pub use swap::*;
pub use types::*;

use std::ffi::c_int;

// =============================================================================
// FFI Exports
// =============================================================================

// Note: rs_ml_line_alloced() is implemented in nvim-buffer crate since it
// accesses buffer state (curbuf->b_ml.ml_flags).

/// Get the ML_EMPTY constant.
///
/// # Safety
/// Pure function, no safety concerns.
#[no_mangle]
pub extern "C" fn rs_ml_get_empty_flag() -> c_int {
    ML_EMPTY
}

/// Get the ML_LINE_DIRTY constant.
///
/// # Safety
/// Pure function, no safety concerns.
#[no_mangle]
pub extern "C" fn rs_ml_get_line_dirty_flag() -> c_int {
    ML_LINE_DIRTY
}

/// Get the ML_LOCKED_DIRTY constant.
///
/// # Safety
/// Pure function, no safety concerns.
#[no_mangle]
pub extern "C" fn rs_ml_get_locked_dirty_flag() -> c_int {
    ML_LOCKED_DIRTY
}

/// Get the ML_LOCKED_POS constant.
///
/// # Safety
/// Pure function, no safety concerns.
#[no_mangle]
pub extern "C" fn rs_ml_get_locked_pos_flag() -> c_int {
    ML_LOCKED_POS
}

/// Get the ML_ALLOCATED constant.
///
/// # Safety
/// Pure function, no safety concerns.
#[no_mangle]
pub extern "C" fn rs_ml_get_allocated_flag() -> c_int {
    ML_ALLOCATED
}

/// Check if an action is "simple" (ML_DELETE, ML_INSERT, or ML_FIND).
///
/// # Safety
/// Pure function, no safety concerns.
#[no_mangle]
pub extern "C" fn rs_ml_is_simple_action(action: c_int) -> c_int {
    c_int::from(ml_simple(action))
}

/// Get the DATA_ID constant for data blocks.
///
/// # Safety
/// Pure function, no safety concerns.
#[no_mangle]
pub extern "C" fn rs_ml_get_data_id() -> u16 {
    DATA_ID
}

/// Get the PTR_ID constant for pointer blocks.
///
/// # Safety
/// Pure function, no safety concerns.
#[no_mangle]
pub extern "C" fn rs_ml_get_ptr_id() -> u16 {
    PTR_ID
}

/// Get the DB_INDEX_MASK constant.
///
/// # Safety
/// Pure function, no safety concerns.
#[no_mangle]
pub extern "C" fn rs_ml_get_db_index_mask() -> u32 {
    DB_INDEX_MASK
}

/// Get the DB_MARKED constant.
///
/// # Safety
/// Pure function, no safety concerns.
#[no_mangle]
pub extern "C" fn rs_ml_get_db_marked() -> u32 {
    DB_MARKED
}

/// Get the BLOCK0_ID0 constant.
///
/// # Safety
/// Pure function, no safety concerns.
#[no_mangle]
pub extern "C" fn rs_ml_get_block0_id0() -> u8 {
    BLOCK0_ID0
}

/// Get the BLOCK0_ID1 constant.
///
/// # Safety
/// Pure function, no safety concerns.
#[no_mangle]
pub extern "C" fn rs_ml_get_block0_id1() -> u8 {
    BLOCK0_ID1
}

/// Get the B0_MAGIC_LONG constant for byte order check.
///
/// # Safety
/// Pure function, no safety concerns.
#[no_mangle]
pub extern "C" fn rs_ml_get_b0_magic_long() -> i64 {
    B0_MAGIC_LONG
}

/// Get the B0_MAGIC_INT constant for byte order check.
///
/// # Safety
/// Pure function, no safety concerns.
#[no_mangle]
pub extern "C" fn rs_ml_get_b0_magic_int() -> i32 {
    B0_MAGIC_INT
}

/// Get the B0_MAGIC_SHORT constant for byte order check.
///
/// # Safety
/// Pure function, no safety concerns.
#[no_mangle]
pub extern "C" fn rs_ml_get_b0_magic_short() -> i16 {
    B0_MAGIC_SHORT
}

/// Get the B0_MAGIC_CHAR constant for byte order check.
///
/// # Safety
/// Pure function, no safety concerns.
#[no_mangle]
pub extern "C" fn rs_ml_get_b0_magic_char() -> u8 {
    B0_MAGIC_CHAR
}

/// Get the B0_DIRTY constant.
///
/// # Safety
/// Pure function, no safety concerns.
#[no_mangle]
pub extern "C" fn rs_ml_get_b0_dirty() -> u8 {
    B0_DIRTY
}

/// Get the B0_FF_MASK constant for fileformat.
///
/// # Safety
/// Pure function, no safety concerns.
#[no_mangle]
pub extern "C" fn rs_ml_get_b0_ff_mask() -> u8 {
    B0_FF_MASK
}

/// Get the B0_SAME_DIR constant.
///
/// # Safety
/// Pure function, no safety concerns.
#[no_mangle]
pub extern "C" fn rs_ml_get_b0_same_dir() -> u8 {
    B0_SAME_DIR
}

/// Get the B0_HAS_FENC constant.
///
/// # Safety
/// Pure function, no safety concerns.
#[no_mangle]
pub extern "C" fn rs_ml_get_b0_has_fenc() -> u8 {
    B0_HAS_FENC
}

/// Get the ML_FIND constant.
///
/// # Safety
/// Pure function, no safety concerns.
#[no_mangle]
pub extern "C" fn rs_ml_get_find_action() -> c_int {
    ML_FIND
}

/// Get the ML_INSERT constant.
///
/// # Safety
/// Pure function, no safety concerns.
#[no_mangle]
pub extern "C" fn rs_ml_get_insert_action() -> c_int {
    ML_INSERT
}

/// Get the ML_DELETE constant.
///
/// # Safety
/// Pure function, no safety concerns.
#[no_mangle]
pub extern "C" fn rs_ml_get_delete_action() -> c_int {
    ML_DELETE
}

/// Get the ML_FLUSH constant.
///
/// # Safety
/// Pure function, no safety concerns.
#[no_mangle]
pub extern "C" fn rs_ml_get_flush_action() -> c_int {
    ML_FLUSH
}

/// Get the ML_CHNK_ADDLINE constant.
///
/// # Safety
/// Pure function, no safety concerns.
#[no_mangle]
pub extern "C" fn rs_ml_get_chnk_addline() -> c_int {
    ML_CHNK_ADDLINE
}

/// Get the ML_CHNK_DELLINE constant.
///
/// # Safety
/// Pure function, no safety concerns.
#[no_mangle]
pub extern "C" fn rs_ml_get_chnk_delline() -> c_int {
    ML_CHNK_DELLINE
}

/// Get the ML_CHNK_UPDLINE constant.
///
/// # Safety
/// Pure function, no safety concerns.
#[no_mangle]
pub extern "C" fn rs_ml_get_chnk_updline() -> c_int {
    ML_CHNK_UPDLINE
}

/// Get the ML_DEL_MESSAGE constant.
///
/// # Safety
/// Pure function, no safety concerns.
#[no_mangle]
pub extern "C" fn rs_ml_get_del_message_flag() -> c_int {
    ML_DEL_MESSAGE
}

/// Get the ML_APPEND_NEW constant.
///
/// # Safety
/// Pure function, no safety concerns.
#[no_mangle]
pub extern "C" fn rs_ml_get_append_new_flag() -> c_int {
    ML_APPEND_NEW
}

/// Get the ML_APPEND_MARK constant.
///
/// # Safety
/// Pure function, no safety concerns.
#[no_mangle]
pub extern "C" fn rs_ml_get_append_mark_flag() -> c_int {
    ML_APPEND_MARK
}

/// Get the B0_FNAME_SIZE_ORG constant.
///
/// # Safety
/// Pure function, no safety concerns.
#[no_mangle]
pub extern "C" fn rs_ml_get_b0_fname_size_org() -> usize {
    B0_FNAME_SIZE_ORG
}

/// Get the B0_FNAME_SIZE_NOCRYPT constant.
///
/// # Safety
/// Pure function, no safety concerns.
#[no_mangle]
pub extern "C" fn rs_ml_get_b0_fname_size_nocrypt() -> usize {
    B0_FNAME_SIZE_NOCRYPT
}

/// Get the B0_FNAME_SIZE_CRYPT constant.
///
/// # Safety
/// Pure function, no safety concerns.
#[no_mangle]
pub extern "C" fn rs_ml_get_b0_fname_size_crypt() -> usize {
    B0_FNAME_SIZE_CRYPT
}

/// Get the B0_UNAME_SIZE constant.
///
/// # Safety
/// Pure function, no safety concerns.
#[no_mangle]
pub extern "C" fn rs_ml_get_b0_uname_size() -> usize {
    B0_UNAME_SIZE
}

/// Get the B0_HNAME_SIZE constant.
///
/// # Safety
/// Pure function, no safety concerns.
#[no_mangle]
pub extern "C" fn rs_ml_get_b0_hname_size() -> usize {
    B0_HNAME_SIZE
}

/// Get the STACK_INCR constant.
///
/// # Safety
/// Pure function, no safety concerns.
#[no_mangle]
pub extern "C" fn rs_ml_get_stack_incr() -> usize {
    STACK_INCR
}

/// Get the INDEX_SIZE constant.
///
/// # Safety
/// Pure function, no safety concerns.
#[no_mangle]
pub extern "C" fn rs_ml_get_index_size() -> usize {
    INDEX_SIZE
}

/// Get the size of the data block header.
///
/// This is the offset where the db_index array starts.
///
/// # Safety
/// Pure function, no safety concerns.
#[no_mangle]
pub extern "C" fn rs_ml_get_data_block_header_size() -> usize {
    DATA_BLOCK_HEADER_SIZE
}

/// Calculate the maximum number of pointer entries for a given page size.
///
/// # Safety
/// Pure function, no safety concerns.
#[no_mangle]
pub extern "C" fn rs_ml_pb_count_max(page_size: usize) -> u16 {
    pb_count_max(page_size)
}

// =============================================================================
// PointerEntry FFI Functions
// =============================================================================

/// Get the block number from a PointerEntry.
///
/// # Safety
/// - `entry` must be a valid pointer to a PointerEntry
#[no_mangle]
pub unsafe extern "C" fn rs_ml_pe_get_bnum(entry: *const PointerEntry) -> BlockNr {
    if entry.is_null() {
        return 0;
    }
    (*entry).pe_bnum
}

/// Set the block number in a PointerEntry.
///
/// # Safety
/// - `entry` must be a valid pointer to a PointerEntry
#[no_mangle]
pub unsafe extern "C" fn rs_ml_pe_set_bnum(entry: *mut PointerEntry, bnum: BlockNr) {
    if !entry.is_null() {
        (*entry).pe_bnum = bnum;
    }
}

/// Get the line count from a PointerEntry.
///
/// # Safety
/// - `entry` must be a valid pointer to a PointerEntry
#[no_mangle]
pub unsafe extern "C" fn rs_ml_pe_get_line_count(entry: *const PointerEntry) -> LineNr {
    if entry.is_null() {
        return 0;
    }
    (*entry).pe_line_count
}

/// Set the line count in a PointerEntry.
///
/// # Safety
/// - `entry` must be a valid pointer to a PointerEntry
#[no_mangle]
pub unsafe extern "C" fn rs_ml_pe_set_line_count(entry: *mut PointerEntry, count: LineNr) {
    if !entry.is_null() {
        (*entry).pe_line_count = count;
    }
}

/// Get the old line number from a PointerEntry.
///
/// # Safety
/// - `entry` must be a valid pointer to a PointerEntry
#[no_mangle]
pub unsafe extern "C" fn rs_ml_pe_get_old_lnum(entry: *const PointerEntry) -> LineNr {
    if entry.is_null() {
        return 0;
    }
    (*entry).pe_old_lnum
}

/// Set the old line number in a PointerEntry.
///
/// # Safety
/// - `entry` must be a valid pointer to a PointerEntry
#[no_mangle]
pub unsafe extern "C" fn rs_ml_pe_set_old_lnum(entry: *mut PointerEntry, lnum: LineNr) {
    if !entry.is_null() {
        (*entry).pe_old_lnum = lnum;
    }
}

/// Get the page count from a PointerEntry.
///
/// # Safety
/// - `entry` must be a valid pointer to a PointerEntry
#[no_mangle]
pub unsafe extern "C" fn rs_ml_pe_get_page_count(entry: *const PointerEntry) -> c_int {
    if entry.is_null() {
        return 0;
    }
    (*entry).pe_page_count
}

/// Set the page count in a PointerEntry.
///
/// # Safety
/// - `entry` must be a valid pointer to a PointerEntry
#[no_mangle]
pub unsafe extern "C" fn rs_ml_pe_set_page_count(entry: *mut PointerEntry, count: c_int) {
    if !entry.is_null() {
        (*entry).pe_page_count = count;
    }
}

// =============================================================================
// PointerBlockHeader FFI Functions
// =============================================================================

/// Check if a pointer block header is valid.
///
/// # Safety
/// - `header` must be a valid pointer to a PointerBlockHeader
#[no_mangle]
pub unsafe extern "C" fn rs_ml_pb_is_valid(header: *const PointerBlockHeader) -> c_int {
    if header.is_null() {
        return 0;
    }
    c_int::from((*header).is_valid())
}

/// Get the count of entries in a pointer block.
///
/// # Safety
/// - `header` must be a valid pointer to a PointerBlockHeader
#[no_mangle]
pub unsafe extern "C" fn rs_ml_pb_get_count(header: *const PointerBlockHeader) -> u16 {
    if header.is_null() {
        return 0;
    }
    (*header).pb_count
}

/// Set the count of entries in a pointer block.
///
/// # Safety
/// - `header` must be a valid pointer to a PointerBlockHeader
#[no_mangle]
pub unsafe extern "C" fn rs_ml_pb_set_count(header: *mut PointerBlockHeader, count: u16) {
    if !header.is_null() {
        (*header).pb_count = count;
    }
}

/// Get the maximum count of entries in a pointer block.
///
/// # Safety
/// - `header` must be a valid pointer to a PointerBlockHeader
#[no_mangle]
pub unsafe extern "C" fn rs_ml_pb_get_count_max(header: *const PointerBlockHeader) -> u16 {
    if header.is_null() {
        return 0;
    }
    (*header).pb_count_max
}

/// Check if a pointer block has room for another entry.
///
/// # Safety
/// - `header` must be a valid pointer to a PointerBlockHeader
#[no_mangle]
pub unsafe extern "C" fn rs_ml_pb_has_room(header: *const PointerBlockHeader) -> c_int {
    if header.is_null() {
        return 0;
    }
    c_int::from((*header).has_room())
}

/// Check if a pointer block is full.
///
/// # Safety
/// - `header` must be a valid pointer to a PointerBlockHeader
#[no_mangle]
pub unsafe extern "C" fn rs_ml_pb_is_full(header: *const PointerBlockHeader) -> c_int {
    if header.is_null() {
        return 1; // Treat null as full
    }
    c_int::from((*header).is_full())
}

// =============================================================================
// DataBlockHeader FFI Functions
// =============================================================================

/// Check if a data block header is valid.
///
/// # Safety
/// - `header` must be a valid pointer to a DataBlockHeader
#[no_mangle]
pub unsafe extern "C" fn rs_ml_db_is_valid(header: *const DataBlockHeader) -> c_int {
    if header.is_null() {
        return 0;
    }
    c_int::from((*header).is_valid())
}

/// Get the free space in a data block.
///
/// # Safety
/// - `header` must be a valid pointer to a DataBlockHeader
#[no_mangle]
pub unsafe extern "C" fn rs_ml_db_get_free(header: *const DataBlockHeader) -> u32 {
    if header.is_null() {
        return 0;
    }
    (*header).db_free
}

/// Set the free space in a data block.
///
/// # Safety
/// - `header` must be a valid pointer to a DataBlockHeader
#[no_mangle]
pub unsafe extern "C" fn rs_ml_db_set_free(header: *mut DataBlockHeader, free: u32) {
    if !header.is_null() {
        (*header).db_free = free;
    }
}

/// Get the text start offset in a data block.
///
/// # Safety
/// - `header` must be a valid pointer to a DataBlockHeader
#[no_mangle]
pub unsafe extern "C" fn rs_ml_db_get_txt_start(header: *const DataBlockHeader) -> u32 {
    if header.is_null() {
        return 0;
    }
    (*header).db_txt_start
}

/// Set the text start offset in a data block.
///
/// # Safety
/// - `header` must be a valid pointer to a DataBlockHeader
#[no_mangle]
pub unsafe extern "C" fn rs_ml_db_set_txt_start(header: *mut DataBlockHeader, start: u32) {
    if !header.is_null() {
        (*header).db_txt_start = start;
    }
}

/// Get the text end offset in a data block.
///
/// # Safety
/// - `header` must be a valid pointer to a DataBlockHeader
#[no_mangle]
pub unsafe extern "C" fn rs_ml_db_get_txt_end(header: *const DataBlockHeader) -> u32 {
    if header.is_null() {
        return 0;
    }
    (*header).db_txt_end
}

/// Get the line count in a data block.
///
/// # Safety
/// - `header` must be a valid pointer to a DataBlockHeader
#[no_mangle]
pub unsafe extern "C" fn rs_ml_db_get_line_count(header: *const DataBlockHeader) -> i64 {
    if header.is_null() {
        return 0;
    }
    (*header).db_line_count
}

/// Set the line count in a data block.
///
/// # Safety
/// - `header` must be a valid pointer to a DataBlockHeader
#[no_mangle]
pub unsafe extern "C" fn rs_ml_db_set_line_count(header: *mut DataBlockHeader, count: i64) {
    if !header.is_null() {
        (*header).db_line_count = count;
    }
}

/// Check if a data block is empty.
///
/// # Safety
/// - `header` must be a valid pointer to a DataBlockHeader
#[no_mangle]
pub unsafe extern "C" fn rs_ml_db_is_empty(header: *const DataBlockHeader) -> c_int {
    if header.is_null() {
        return 1; // Treat null as empty
    }
    c_int::from((*header).is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant_exports() {
        // Verify exported constants match internal values
        assert_eq!(rs_ml_get_empty_flag(), ML_EMPTY);
        assert_eq!(rs_ml_get_line_dirty_flag(), ML_LINE_DIRTY);
        assert_eq!(rs_ml_get_data_id(), DATA_ID);
        assert_eq!(rs_ml_get_ptr_id(), PTR_ID);
    }

    #[test]
    fn test_data_block_header_size() {
        let size = rs_ml_get_data_block_header_size();
        // Should be at least the size of the fields (2 + 4 + 4 + 4 + 8 = 22 bytes)
        // but may be larger due to alignment
        assert!(size >= 22);
    }

    #[test]
    fn test_pb_count_max_export() {
        let count = rs_ml_pb_count_max(4096);
        assert!(count > 100);
        assert!(count < 200);
    }

    #[test]
    fn test_pointer_entry_ffi() {
        let mut entry = PointerEntry::new();

        unsafe {
            assert_eq!(rs_ml_pe_get_bnum(&raw const entry), 0);
            assert_eq!(rs_ml_pe_get_line_count(&raw const entry), 0);
            assert_eq!(rs_ml_pe_get_old_lnum(&raw const entry), 0);
            assert_eq!(rs_ml_pe_get_page_count(&raw const entry), 0);

            rs_ml_pe_set_bnum(&raw mut entry, 42);
            rs_ml_pe_set_line_count(&raw mut entry, 100);
            rs_ml_pe_set_old_lnum(&raw mut entry, 1);
            rs_ml_pe_set_page_count(&raw mut entry, 3);

            assert_eq!(rs_ml_pe_get_bnum(&raw const entry), 42);
            assert_eq!(rs_ml_pe_get_line_count(&raw const entry), 100);
            assert_eq!(rs_ml_pe_get_old_lnum(&raw const entry), 1);
            assert_eq!(rs_ml_pe_get_page_count(&raw const entry), 3);
        }
    }

    #[test]
    fn test_pointer_block_header_ffi() {
        let mut header = PointerBlockHeader::new(128);

        unsafe {
            assert_eq!(rs_ml_pb_is_valid(&raw const header), 1);
            assert_eq!(rs_ml_pb_get_count(&raw const header), 0);
            assert_eq!(rs_ml_pb_get_count_max(&raw const header), 128);
            assert_eq!(rs_ml_pb_has_room(&raw const header), 1);
            assert_eq!(rs_ml_pb_is_full(&raw const header), 0);

            rs_ml_pb_set_count(&raw mut header, 128);
            assert_eq!(rs_ml_pb_get_count(&raw const header), 128);
            assert_eq!(rs_ml_pb_has_room(&raw const header), 0);
            assert_eq!(rs_ml_pb_is_full(&raw const header), 1);
        }
    }

    #[test]
    fn test_data_block_header_ffi() {
        let mut header = DataBlockHeader::new(4096);

        unsafe {
            assert_eq!(rs_ml_db_is_valid(&raw const header), 1);
            assert_eq!(rs_ml_db_get_txt_start(&raw const header), 4096);
            assert_eq!(rs_ml_db_get_txt_end(&raw const header), 4096);
            assert_eq!(rs_ml_db_get_line_count(&raw const header), 0);
            assert_eq!(rs_ml_db_is_empty(&raw const header), 1);

            rs_ml_db_set_line_count(&raw mut header, 10);
            assert_eq!(rs_ml_db_get_line_count(&raw const header), 10);
            assert_eq!(rs_ml_db_is_empty(&raw const header), 0);

            rs_ml_db_set_txt_start(&raw mut header, 3000);
            assert_eq!(rs_ml_db_get_txt_start(&raw const header), 3000);

            rs_ml_db_set_free(&raw mut header, 500);
            assert_eq!(rs_ml_db_get_free(&raw const header), 500);
        }
    }
}
