//! Core data structures and constants for the memline system.
//!
//! The memline system manages buffer text in a B-tree structure with pointer blocks
//! (internal nodes) and data blocks (leaf nodes). Block 0 contains recovery information.
//!
//! # Block Types
//!
//! - **Block 0**: Contains swap file metadata for crash recovery
//! - **Pointer blocks**: Internal tree nodes with pointers to child blocks
//! - **Data blocks**: Leaf nodes containing actual line text
//!
//! # Memory Layout
//!
//! Data blocks store lines in reverse order - the first line's text is at the end
//! of the block, with subsequent lines placed before it.

use std::ffi::c_int;

// =============================================================================
// Block Type Constants
// =============================================================================

/// ID for data blocks: ('d' << 8) + 'a' = 0x6461
pub const DATA_ID: u16 = (b'd' as u16) << 8 | (b'a' as u16);

/// ID for pointer blocks: ('p' << 8) + 't' = 0x7074
pub const PTR_ID: u16 = (b'p' as u16) << 8 | (b't' as u16);

/// Block 0 ID byte 0
pub const BLOCK0_ID0: u8 = b'b';

/// Block 0 ID byte 1
pub const BLOCK0_ID1: u8 = b'0';

// =============================================================================
// Memory Line Flags
// =============================================================================

/// Buffer is empty (only contains one empty line)
pub const ML_EMPTY: c_int = 0x01;

/// Cached line was changed and allocated
pub const ML_LINE_DIRTY: c_int = 0x02;

/// Locked block (`ml_locked`) was changed
pub const ML_LOCKED_DIRTY: c_int = 0x04;

/// Locked block needs positive block number
pub const ML_LOCKED_POS: c_int = 0x08;

/// `ml_line_ptr` is an allocated copy
pub const ML_ALLOCATED: c_int = 0x10;

// =============================================================================
// Block 0 Constants
// =============================================================================

/// Original size of filename in block 0
pub const B0_FNAME_SIZE_ORG: usize = 900;

/// Filename size when no encryption (2 bytes used for other things)
pub const B0_FNAME_SIZE_NOCRYPT: usize = 898;

/// Filename size with encryption (10 bytes used for other things)
pub const B0_FNAME_SIZE_CRYPT: usize = 890;

/// Size of username field in block 0
pub const B0_UNAME_SIZE: usize = 40;

/// Size of hostname field in block 0
pub const B0_HNAME_SIZE: usize = 40;

/// Magic number for long type in block 0 (byte order check)
pub const B0_MAGIC_LONG: i64 = 0x3031_3233;

/// Magic number for int type in block 0 (byte order check)
pub const B0_MAGIC_INT: i32 = 0x2021_2223;

/// Magic number for short type in block 0 (byte order check)
/// This value is intentionally 0x1213 (truncated from 0x10111213).
pub const B0_MAGIC_SHORT: i16 = 0x1213;

/// Magic character in block 0 (byte order check)
pub const B0_MAGIC_CHAR: u8 = 0x55;

/// Dirty flag value in block 0
pub const B0_DIRTY: u8 = 0x55;

/// Fileformat mask in b0_flags (lowest 2 bits)
pub const B0_FF_MASK: u8 = 3;

/// Flag: swapfile is in same directory as edited file
pub const B0_SAME_DIR: u8 = 4;

/// Flag: fileencoding is stored at end of b0_fname
pub const B0_HAS_FENC: u8 = 8;

// =============================================================================
// Tree Navigation Constants
// =============================================================================

/// Number of entries added to ml_stack at a time
pub const STACK_INCR: usize = 5;

/// Action: delete line
pub const ML_DELETE: c_int = 0x11;

/// Action: insert line
pub const ML_INSERT: c_int = 0x12;

/// Action: just find the line
pub const ML_FIND: c_int = 0x13;

/// Action: flush locked block
pub const ML_FLUSH: c_int = 0x02;

/// Check if action is simple (DEL, INS, or FIND)
#[inline]
#[must_use]
pub const fn ml_simple(action: c_int) -> bool {
    (action & 0x10) != 0
}

// =============================================================================
// Data Block Constants
// =============================================================================

/// High bit of db_index used for marking lines (global command)
pub const DB_MARKED: u32 = 1 << (std::mem::size_of::<u32>() * 8 - 1);

/// Mask to get actual index from db_index
pub const DB_INDEX_MASK: u32 = !DB_MARKED;

/// Size of one db_index entry
pub const INDEX_SIZE: usize = std::mem::size_of::<u32>();

// =============================================================================
// Chunk Constants (for byte offset caching)
// =============================================================================

/// Update chunk: adding a line
pub const ML_CHNK_ADDLINE: c_int = 1;

/// Update chunk: deleting a line
pub const ML_CHNK_DELLINE: c_int = 2;

/// Update chunk: updating a line
pub const ML_CHNK_UPDLINE: c_int = 3;

// =============================================================================
// Delete/Append Flags
// =============================================================================

/// May give a "No lines in buffer" message
pub const ML_DEL_MESSAGE: c_int = 1;

/// Starting to edit a new file
pub const ML_APPEND_NEW: c_int = 1;

/// Mark the new line
pub const ML_APPEND_MARK: c_int = 2;

// =============================================================================
// Block 0 Update Types
// =============================================================================

/// Update timestamp and filename
pub const UB_FNAME: c_int = 0;

/// Update the B0_SAME_DIR flag
pub const UB_SAME_DIR: c_int = 1;

// =============================================================================
// Swap Existence Action Choices
// =============================================================================

/// No choice made yet
pub const SEA_CHOICE_NONE: c_int = 0;

/// Open file readonly
pub const SEA_CHOICE_READONLY: c_int = 1;

/// Edit anyway (ignore swap)
pub const SEA_CHOICE_EDIT: c_int = 2;

/// Recover from swap file
pub const SEA_CHOICE_RECOVER: c_int = 3;

/// Delete the swap file
pub const SEA_CHOICE_DELETE: c_int = 4;

/// Quit
pub const SEA_CHOICE_QUIT: c_int = 5;

/// Abort
pub const SEA_CHOICE_ABORT: c_int = 6;

// =============================================================================
// Opaque Handles for C Types
// =============================================================================

/// Opaque handle to `memfile_T` in C.
///
/// The memfile is the underlying block storage for the memline B-tree.
/// It manages reading/writing blocks to the swap file.
#[repr(C)]
pub struct MemfileHandle {
    _private: [u8; 0],
}

/// Opaque handle to `bhdr_T` (block header) in C.
///
/// Block headers track individual blocks in the memfile, including
/// their data pointer, page count, and dirty/locked status.
#[repr(C)]
pub struct BlockHeaderHandle {
    _private: [u8; 0],
}

/// Opaque handle to `buf_T` (buffer) in C.
///
/// This represents a Neovim buffer with its memline and other metadata.
#[repr(C)]
pub struct BufHandle {
    _private: [u8; 0],
}

/// Opaque handle to `memline_T` in C.
///
/// The memline structure contains the B-tree for buffer text storage,
/// including the root pointer, cached line, and chunk information.
#[repr(C)]
pub struct MemlineHandle {
    _private: [u8; 0],
}

/// Opaque handle to `infoptr_T` (tree stack entry) in C.
///
/// Stack entries track the path from root to a data block during tree traversal.
#[repr(C)]
pub struct InfoPtrHandle {
    _private: [u8; 0],
}

/// Opaque handle to `DataBlock` in C.
///
/// Data blocks are leaf nodes in the B-tree containing actual line text.
#[repr(C)]
pub struct DataBlockHandle {
    _private: [u8; 0],
}

/// Opaque handle to `PointerBlock` in C.
///
/// Pointer blocks are internal nodes containing pointers to child blocks.
#[repr(C)]
pub struct PointerBlockHandle {
    _private: [u8; 0],
}

/// Opaque handle to `ZeroBlock` (block 0) in C.
///
/// Block 0 contains swap file metadata for crash recovery.
#[repr(C)]
pub struct ZeroBlockHandle {
    _private: [u8; 0],
}

/// Opaque handle to `chunksize_T` in C.
///
/// Chunks cache line counts and byte sizes for fast line/byte conversion.
#[repr(C)]
pub struct ChunkSizeHandle {
    _private: [u8; 0],
}

/// Opaque handle to `pos_T` (cursor position) in C.
#[repr(C)]
pub struct PosHandle {
    _private: [u8; 0],
}

// =============================================================================
// Type Aliases for FFI
// =============================================================================

/// Line number type (signed, 1-based)
pub type LineNr = i64;

/// Block number type (signed, can be negative for unassigned blocks)
pub type BlockNr = i64;

/// Column number type
pub type ColNr = i32;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_ids() {
        // Verify block IDs match the C definitions
        assert_eq!(DATA_ID, 0x6461);
        assert_eq!(PTR_ID, 0x7074);
        assert_eq!(BLOCK0_ID0, b'b');
        assert_eq!(BLOCK0_ID1, b'0');
    }

    #[test]
    fn test_ml_flags() {
        assert_eq!(ML_EMPTY, 0x01);
        assert_eq!(ML_LINE_DIRTY, 0x02);
        assert_eq!(ML_LOCKED_DIRTY, 0x04);
        assert_eq!(ML_LOCKED_POS, 0x08);
        assert_eq!(ML_ALLOCATED, 0x10);
    }

    #[test]
    fn test_ml_simple() {
        assert!(ml_simple(ML_DELETE));
        assert!(ml_simple(ML_INSERT));
        assert!(ml_simple(ML_FIND));
        assert!(!ml_simple(ML_FLUSH));
    }

    #[test]
    fn test_magic_numbers() {
        // These magic numbers are used for byte-order detection
        assert_eq!(B0_MAGIC_LONG, 0x3031_3233);
        assert_eq!(B0_MAGIC_INT, 0x2021_2223);
        assert_eq!(B0_MAGIC_SHORT, 0x1213);
        assert_eq!(B0_MAGIC_CHAR, 0x55);
    }

    #[test]
    fn test_db_index_mask() {
        // The mask should clear only the top bit
        let marked = DB_MARKED | 0x1234;
        assert_eq!(marked & DB_INDEX_MASK, 0x1234);
    }
}
