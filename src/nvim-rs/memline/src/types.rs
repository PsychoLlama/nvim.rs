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
// Buffer Flags (b_flags field)
// =============================================================================

/// Buffer has been recovered
pub const BF_RECOVERED: c_int = 0x01;

// =============================================================================
// Block 0 Update Types
// =============================================================================

/// Update timestamp and filename
pub const UB_FNAME: c_int = 0;

/// Update the B0_SAME_DIR flag
pub const UB_SAME_DIR: c_int = 1;

// =============================================================================
// General Result Codes
// =============================================================================

/// Function succeeded
pub const OK: c_int = 1;

/// Function failed
pub const FAIL: c_int = 0;

// =============================================================================
// Memfile Sync Flags
// =============================================================================

/// Sync flag: also sync blocks with negative numbers
pub const MFS_ALL: c_int = 1;

/// Sync flag: stop syncing when a character is available
pub const MFS_STOP: c_int = 2;

/// Sync flag: flush file to disk (fsync)
pub const MFS_FLUSH: c_int = 4;

/// Sync flag: only write block 0
pub const MFS_ZERO: c_int = 8;

// =============================================================================
// Swap Existence Action (swap_exists_action global values)
// =============================================================================

/// Don't use dialog (SEA_NONE)
pub const SEA_NONE: c_int = 0;

/// Quit editing the file (SEA_QUIT)
pub const SEA_QUIT: c_int = 2;

/// Recover the file (SEA_RECOVER)
pub const SEA_RECOVER: c_int = 3;

/// No dialog, mark buffer as read-only (SEA_READONLY)
pub const SEA_READONLY: c_int = 4;

// =============================================================================
// Buffer flags (b_flags field, additional values)
// =============================================================================

/// Dummy buffer used for autocommands (BF_DUMMY)
pub const BF_DUMMY: c_int = 0x08;

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

/// Concrete mirror of `infoptr_T` for pointer arithmetic on `ml_stack` arrays.
///
/// Layout (24 bytes, 8-byte aligned):
/// - `ip_bnum`:  blocknr_T (i64), offset 0
/// - `ip_low`:   linenr_T (i32), offset 8
/// - `ip_high`:  linenr_T (i32), offset 12
/// - `ip_index`: int (i32),      offset 16
/// - 4 bytes trailing padding
#[repr(C)]
pub struct InfoPtr {
    pub ip_bnum: i64,
    pub ip_low: i32,
    pub ip_high: i32,
    pub ip_index: std::ffi::c_int,
    _pad: [u8; 4],
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

/// Repr(C) mirror of `chunksize_T` from C.
///
/// Used for direct indexed access into `buf->b_ml.ml_chunksize[idx]`.
/// Layout: two `c_int` fields = 8 bytes total, matching `sizeof(chunksize_T) == 8`.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ChunkSizeEntry {
    /// Number of lines in this chunk (`mlcs_numlines`)
    pub mlcs_numlines: c_int,
    /// Total byte size of all lines in this chunk (`mlcs_totalsize`)
    pub mlcs_totalsize: c_int,
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

// =============================================================================
// B-tree Data Structures
// =============================================================================

/// Entry in a pointer block, representing a branch in the B-tree.
///
/// Each entry points to a child block (either another pointer block or a data block)
/// and tracks the number of lines in that branch.
///
/// # Fields (mirrored from C)
///
/// - `pe_bnum` - Block number of the child
/// - `pe_line_count` - Total number of lines in this branch
/// - `pe_old_lnum` - Line number for this block (used during recovery)
/// - `pe_page_count` - Number of pages in the child block
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct PointerEntry {
    /// Block number of the child block
    pub pe_bnum: BlockNr,
    /// Number of lines in this branch.
    /// C type: `linenr_T = int32_t` (4 bytes).  Must NOT be widened to i64
    /// here, as the field is part of a `#[repr(C)]` struct that must match
    /// the C `PointerEntry` layout exactly (sizeof = 24).
    pub pe_line_count: i32,
    /// Line number for recovery.
    /// C type: `linenr_T = int32_t` (4 bytes).
    pub pe_old_lnum: i32,
    /// Number of pages in the child block
    pub pe_page_count: c_int,
}

impl PointerEntry {
    /// Create a new empty pointer entry.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            pe_bnum: 0,
            pe_line_count: 0,
            pe_old_lnum: 0,
            pe_page_count: 0,
        }
    }

    /// Create a pointer entry with the given values.
    #[must_use]
    pub const fn with_values(
        bnum: BlockNr,
        line_count: i32,
        old_lnum: i32,
        page_count: c_int,
    ) -> Self {
        Self {
            pe_bnum: bnum,
            pe_line_count: line_count,
            pe_old_lnum: old_lnum,
            pe_page_count: page_count,
        }
    }
}

/// Header for a pointer block (internal B-tree node).
///
/// Pointer blocks contain an array of `PointerEntry` values, each pointing
/// to a child block. The actual pointer array follows immediately after
/// this header in memory.
///
/// # Memory Layout
///
/// ```text
/// +------------------+
/// | pb_id (u16)      |  Block ID: PTR_ID (0x7074)
/// | pb_count (u16)   |  Number of valid entries
/// | pb_count_max(u16)|  Maximum entries that fit
/// +------------------+
/// | pb_pointer[0]    |  First PointerEntry
/// | pb_pointer[1]    |  Second PointerEntry
/// | ...              |
/// | pb_pointer[n-1]  |  Last PointerEntry
/// +------------------+
/// | (empty space)    |  Unused space to end of page
/// +------------------+
/// ```
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PointerBlockHeader {
    /// Block ID, should be PTR_ID (0x7074)
    pub pb_id: u16,
    /// Number of valid pointer entries
    pub pb_count: u16,
    /// Maximum number of entries that fit in this block
    pub pb_count_max: u16,
    /// Padding to match C layout: `pb_pointer[]` starts at offset 8
    /// (C compiler inserts 2 bytes here for PointerEntry's 8-byte alignment)
    _pad: u16,
}

impl PointerBlockHeader {
    /// Create a new pointer block header.
    #[must_use]
    pub const fn new(count_max: u16) -> Self {
        Self {
            pb_id: PTR_ID,
            pb_count: 0,
            pb_count_max: count_max,
            _pad: 0,
        }
    }

    /// Check if this is a valid pointer block.
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.pb_id == PTR_ID
    }

    /// Check if there's room for another entry.
    #[must_use]
    pub const fn has_room(&self) -> bool {
        self.pb_count < self.pb_count_max
    }

    /// Check if the block is full.
    #[must_use]
    pub const fn is_full(&self) -> bool {
        self.pb_count >= self.pb_count_max
    }
}

impl Default for PointerBlockHeader {
    fn default() -> Self {
        Self {
            pb_id: PTR_ID,
            pb_count: 0,
            pb_count_max: 0,
            _pad: 0,
        }
    }
}

/// Header for a data block (B-tree leaf node).
///
/// Data blocks store the actual line text. Lines are stored in reverse order -
/// the first line's text is at the end of the block, with subsequent lines
/// placed before it.
///
/// # Memory Layout
///
/// ```text
/// +------------------+
/// | db_id (u16)      |  Block ID: DATA_ID (0x6461)
/// | db_free (u32)    |  Free space available
/// | db_txt_start(u32)|  Byte offset where text starts
/// | db_txt_end (u32) |  Byte offset just after block
/// | db_line_count    |  Number of lines in this block
/// +------------------+
/// | db_index[0]      |  Offset of first line's text
/// | db_index[1]      |  Offset of second line's text
/// | ...              |
/// +------------------+
/// | (free space)     |  Empty space between indexes and text
/// +------------------+
/// | line N text      |  Text of last line (near db_txt_start)
/// | ...              |
/// | line 1 text      |  Text of first line (at end of block)
/// +------------------+
/// ```
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DataBlockHeader {
    /// Block ID, should be DATA_ID (0x6461)
    pub db_id: u16,
    /// Free space available in the block
    pub db_free: u32,
    /// Byte offset where text starts (grows downward)
    pub db_txt_start: u32,
    /// Byte offset just after the block (block size)
    pub db_txt_end: u32,
    /// Number of lines stored in this block
    pub db_line_count: i64,
}

impl DataBlockHeader {
    /// Create a new data block header for a block of the given size.
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub const fn new(block_size: u32) -> Self {
        // Header size is the offset of db_index (where the index array starts)
        // Safe: header size is always small (< 32 bytes)
        let header_size = std::mem::size_of::<Self>() as u32;
        Self {
            db_id: DATA_ID,
            db_free: block_size - header_size,
            db_txt_start: block_size,
            db_txt_end: block_size,
            db_line_count: 0,
        }
    }

    /// Check if this is a valid data block.
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.db_id == DATA_ID
    }

    /// Check if the block is empty (no lines).
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.db_line_count == 0
    }
}

impl Default for DataBlockHeader {
    fn default() -> Self {
        Self {
            db_id: DATA_ID,
            db_free: 0,
            db_txt_start: 0,
            db_txt_end: 0,
            db_line_count: 0,
        }
    }
}

/// Size of the data block header (offset to db_index array).
pub const DATA_BLOCK_HEADER_SIZE: usize = std::mem::size_of::<DataBlockHeader>();

// =============================================================================
// Additional constants for C accessor elimination
// =============================================================================

/// Minimum swap page size (MIN_SWAP_PAGE_SIZE in memfile.h)
pub const MIN_SWAP_PAGE_SIZE: u32 = 1048;

/// HLF_E highlight group (error messages) - index in hlf_T enum
/// HLF_NONE=0, HLF_8=1, HLF_EOB=2, HLF_TERM=3, HLF_AT=4, HLF_D=5, HLF_E=6
pub const HLF_E: c_int = 6;

/// SHM_ATTENTION: 'A' - No ATTENTION messages in 'shortmess'
pub const SHM_ATTENTION: c_int = b'A' as c_int;

/// UPD_NOT_VALID: buffer needs complete redraw (value 40)
pub const UPD_NOT_VALID: c_int = 40;

/// Size of b0_version field in ZeroBlock (sizeof(b0_version) = 10)
pub const B0_VERSION_SIZE: usize = 10;

/// Calculate the maximum number of pointer entries that fit in a page.
///
/// # Arguments
/// * `page_size` - Size of a memory page in bytes
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub const fn pb_count_max(page_size: usize) -> u16 {
    let header_size = std::mem::size_of::<PointerBlockHeader>();
    let entry_size = std::mem::size_of::<PointerEntry>();
    // Safe: result is always a small number (< 200 for typical page sizes)
    ((page_size - header_size) / entry_size) as u16
}

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

    #[test]
    fn test_pointer_entry() {
        let entry = PointerEntry::new();
        assert_eq!(entry.pe_bnum, 0);
        assert_eq!(entry.pe_line_count, 0);
        assert_eq!(entry.pe_old_lnum, 0);
        assert_eq!(entry.pe_page_count, 0);

        let entry = PointerEntry::with_values(42, 100, 1, 3);
        assert_eq!(entry.pe_bnum, 42);
        assert_eq!(entry.pe_line_count, 100);
        assert_eq!(entry.pe_old_lnum, 1);
        assert_eq!(entry.pe_page_count, 3);
    }

    #[test]
    fn test_layout_compat() {
        // PointerEntry: pe_bnum(i64,8) + pe_line_count(i32,4) + pe_old_lnum(i32,4)
        //             + pe_page_count(i32,4) + padding(4) = 24 bytes (matches C)
        assert_eq!(std::mem::size_of::<PointerEntry>(), 24);
        // PointerBlockHeader: pb_id(2) + pb_count(2) + pb_count_max(2) + _pad(2) = 8 bytes
        assert_eq!(std::mem::size_of::<PointerBlockHeader>(), 8);
        // DataBlockHeader: db_id(2)+pad(2) + db_free(4) + db_txt_start(4) + db_txt_end(4)
        //                + pad(4? no) + db_line_count(long=8) = 24 bytes (matches C)
        assert_eq!(std::mem::size_of::<DataBlockHeader>(), 24);
    }

    #[test]
    fn test_pointer_block_header() {
        let header = PointerBlockHeader::new(128);
        assert!(header.is_valid());
        assert_eq!(header.pb_id, PTR_ID);
        assert_eq!(header.pb_count, 0);
        assert_eq!(header.pb_count_max, 128);
        assert!(header.has_room());
        assert!(!header.is_full());
    }

    #[test]
    fn test_data_block_header() {
        let header = DataBlockHeader::new(4096);
        assert!(header.is_valid());
        assert_eq!(header.db_id, DATA_ID);
        assert_eq!(header.db_txt_end, 4096);
        assert_eq!(header.db_txt_start, 4096);
        assert!(header.is_empty());
    }

    #[test]
    fn test_pb_count_max() {
        // For a 4096-byte page, header is 8 bytes (3 u16 + 2 bytes padding),
        // entry is 32 bytes (8+8+8+4+4 padding).
        // (4096 - 8) / 32 = 127 entries
        let count = pb_count_max(4096);
        assert!(count > 100);
        assert!(count < 200);
    }
}
