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
//! - [`types`]: Core data structures, constants, and opaque handles
//! - [`access`]: Line access functions
//! - [`modify`]: Line modification functions (append, delete, replace)
//! - [`navigate`]: Cursor navigation and byte offset functions
//! - [`swap`]: Swap file management
//! - [`recovery`]: Recovery and attention file handling
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
pub mod modify;
pub mod navigate;
pub mod recovery;
pub mod swap;
pub mod types;

// Re-export all public items for FFI
pub use access::*;
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
}
