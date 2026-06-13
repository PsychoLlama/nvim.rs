//! Buffer header implementation for stuff/redo/recording buffers
//!
//! This module provides safe Rust wrappers for the `buffheader_T` and
//! `buffblock_T` types used for managing the stuff buffer, redo buffer,
//! and recording buffer.
//!
//! The 5 global buffer statics (redobuff, old_redobuff, recordbuff,
//! readbuf1, readbuf2) are owned here, along with the block_redo flag.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr,
    clippy::items_after_statements,
    static_mut_refs
)]

use std::ffi::c_int;

use crate::stuff;

/// Minimum size for a buffer block's string content.
const MINIMAL_SIZE: usize = 20;

// Key encoding constants (must match keycodes.h)
const K_SPECIAL: u8 = 0x80;
const KS_SPECIAL: u8 = 254;
const KS_ZERO: u8 = 255;
const KE_FILLER: u8 = b'X';

// =============================================================================
// Global Buffer Statics
// =============================================================================

static mut REDOBUFF: BuffHeader = BuffHeader::new();
static mut OLD_REDOBUFF: BuffHeader = BuffHeader::new();
static mut RECORDBUFF: BuffHeader = BuffHeader::new();
static mut READBUF1: BuffHeader = BuffHeader::new();
static mut READBUF2: BuffHeader = BuffHeader::new();
static mut BLOCK_REDO: bool = false;

/// A block in a buffer chain.
///
/// Each block contains a string segment and a pointer to the next block.
/// This forms a linked list of string segments that can be efficiently
/// appended to and read from.
#[derive(Debug)]
pub struct BuffBlock {
    /// The string content of this block
    content: Vec<u8>,
}

impl BuffBlock {
    /// Create a new buffer block with the given content.
    #[must_use]
    pub fn new(content: &[u8]) -> Self {
        Self {
            content: content.to_vec(),
        }
    }

    /// Create a new buffer block with reserved capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            content: Vec::with_capacity(capacity),
        }
    }

    /// Get the string content of this block.
    #[must_use]
    pub const fn content(&self) -> &Vec<u8> {
        &self.content
    }

    /// Get the length of the string content.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.content.len()
    }

    /// Check if the block is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    /// Append content to this block.
    pub fn append(&mut self, s: &[u8]) {
        self.content.extend_from_slice(s);
    }

    /// Get remaining capacity (for blocks with reserved space).
    #[must_use]
    pub const fn remaining_capacity(&self) -> usize {
        self.content.capacity() - self.content.len()
    }

    /// Truncate the content by removing the last `n` bytes.
    pub fn truncate_tail(&mut self, n: usize) {
        if n <= self.content.len() {
            self.content.truncate(self.content.len() - n);
        }
    }
}

/// Header for a buffer chain.
///
/// This manages a linked list of `BuffBlock`s, used for:
/// - `readbuf1`: Translated commands (stuff buffer)
/// - `readbuf2`: Redo buffer
/// - `redobuff`: The redo buffer
/// - `old_redobuff`: Previous redo buffer
/// - `recordbuff`: Recording buffer
///
/// The buffer supports efficient appending at the end and reading from
/// the beginning.
#[derive(Debug, Default)]
pub struct BuffHeader {
    /// First block in the chain (blocks are owned)
    blocks: Vec<BuffBlock>,
    /// Index of the current block for appending
    curr_block_idx: Option<usize>,
    /// Index for reading within the first block
    read_index: usize,
    /// Space remaining in current block for appending
    space: usize,
    /// Whether to create a new block on next append
    create_newblock: bool,
}

impl BuffHeader {
    /// Create a new, empty buffer header.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            blocks: Vec::new(),
            curr_block_idx: None,
            read_index: 0,
            space: 0,
            create_newblock: false,
        }
    }

    /// Check if the buffer is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }

    /// Prepare the buffer for reading (after stuffing).
    pub const fn start_reading(&mut self) {
        if !self.blocks.is_empty() {
            self.curr_block_idx = Some(0);
            self.create_newblock = true;
        }
    }

    /// Add a string to the end of the buffer.
    ///
    /// K_SPECIAL should have been escaped already.
    pub fn add(&mut self, s: &[u8]) {
        if s.is_empty() {
            return;
        }

        if self.blocks.is_empty() {
            // First add to list
            self.curr_block_idx = Some(0);
            self.create_newblock = true;
        } else if self.curr_block_idx.is_none() {
            // Buffer has already been read - this is an error condition
            // In C this would call iemsg("E222: Add to read buffer")
            return;
        } else if self.read_index != 0 {
            // Compact the first block by removing already-read content
            if let Some(first) = self.blocks.first_mut() {
                let remaining = first.content[self.read_index..].to_vec();
                first.content = remaining;
                self.space += self.read_index;
            }
            self.read_index = 0;
        }

        if !self.create_newblock && self.space >= s.len() {
            // Append to current block
            if let Some(idx) = self.curr_block_idx {
                if let Some(block) = self.blocks.get_mut(idx) {
                    block.append(s);
                    self.space -= s.len();
                }
            }
        } else {
            // Create a new block
            let len = MINIMAL_SIZE.max(s.len());
            let mut new_block = BuffBlock::with_capacity(len);
            new_block.append(s);
            self.space = len - s.len();
            self.create_newblock = false;

            self.blocks.push(new_block);
            self.curr_block_idx = Some(self.blocks.len() - 1);
        }
    }

    /// Delete `n` bytes from the end of the buffer.
    ///
    /// Only works when content was just added.
    pub fn delete_tail(&mut self, n: usize) {
        if let Some(idx) = self.curr_block_idx {
            if let Some(block) = self.blocks.get_mut(idx) {
                if block.len() >= n {
                    block.truncate_tail(n);
                    self.space += n;
                }
            }
        }
    }

    /// Read one byte from the buffer.
    ///
    /// If `advance` is true, move the read position forward.
    ///
    /// Returns `None` if the buffer is empty.
    pub fn read_byte(&mut self, advance: bool) -> Option<u8> {
        if self.blocks.is_empty() {
            return None;
        }

        let first = self.blocks.first()?;
        if self.read_index >= first.len() {
            return None;
        }

        let c = first.content[self.read_index];

        if advance {
            self.read_index += 1;
            // Check if we've consumed the entire first block
            if self.read_index >= first.len() {
                self.blocks.remove(0);
                self.read_index = 0;
                // Update curr_block_idx
                if self.blocks.is_empty() {
                    self.curr_block_idx = None;
                } else if let Some(idx) = self.curr_block_idx {
                    if idx > 0 {
                        self.curr_block_idx = Some(idx - 1);
                    }
                }
            }
        }

        Some(c)
    }

    /// Get all content as a single string.
    ///
    /// K_SPECIAL in the returned string is escaped.
    #[must_use]
    pub fn get_contents(&self) -> Vec<u8> {
        let total_len: usize = self.blocks.iter().map(BuffBlock::len).sum();
        let mut result = Vec::with_capacity(total_len);

        for (i, block) in self.blocks.iter().enumerate() {
            let start = if i == 0 { self.read_index } else { 0 };
            result.extend_from_slice(&block.content[start..]);
        }

        result
    }

    /// Clear the buffer and free all blocks.
    pub fn clear(&mut self) {
        self.blocks.clear();
        self.curr_block_idx = None;
        self.read_index = 0;
        self.space = 0;
        self.create_newblock = false;
    }

    /// Add a number to the buffer as a string.
    pub fn add_num(&mut self, n: i32) {
        let s = n.to_string();
        self.add(s.as_bytes());
    }

    /// Add a single byte to the buffer.
    ///
    /// Translates special keys, NUL, and K_SPECIAL into 3-byte sequences.
    pub fn add_byte(&mut self, c: u8) {
        if c == K_SPECIAL || c == 0 {
            let ks = if c == K_SPECIAL { KS_SPECIAL } else { KS_ZERO };
            self.add(&[K_SPECIAL, ks, KE_FILLER]);
        } else {
            self.add(&[c]);
        }
    }

    /// Add a character to the buffer (like C's `add_char_buff` in C Neovim).
    ///
    /// Mirrors the C implementation exactly:
    ///
    /// - **Special keys** (negative `c`, i.e. `IS_SPECIAL(c)`) and the literal
    ///   byte values `K_SPECIAL` (0x80) and `NUL` (0x00): emit a 3-byte escape
    ///   sequence `[K_SPECIAL, K_SECOND(c), K_THIRD(c)]` **verbatim** via
    ///   `add()`.  These three bytes are already in final encoded form.
    ///
    /// - **Non-special characters** (positive codepoints, including ASCII and
    ///   multibyte Unicode): convert to UTF-8 bytes, then pass each byte through
    ///   `add_byte()`, which escapes any 0x80 or NUL content byte as
    ///   `[K_SPECIAL, KS_SPECIAL, KE_FILLER]` / `[K_SPECIAL, KS_ZERO, KE_FILLER]`.
    ///
    /// The previous (buggy) implementation called `add_byte` on ALL bytes of
    /// `encode_char` output, which double-escaped the leading 0x80 of special-key
    /// sequences, corrupting `K_PASTE_START`/`K_PASTE_END` in redo/record buffers.
    pub fn add_char(&mut self, c: c_int) {
        let mut buf = [0u8; stuff::CHAR_BUF_SIZE];
        let len = stuff::encode_char(c, &mut buf);
        if stuff::is_special(c) || c == c_int::from(K_SPECIAL) || c == 0 {
            // Special key or literal K_SPECIAL/NUL: encode_char already produces
            // the final 3-byte sequence [K_SPECIAL, K_SECOND, K_THIRD]; write verbatim.
            self.add(&buf[..len]);
        } else {
            // Non-special: UTF-8 bytes — each byte may be 0x80 (content) and must
            // be escaped through add_byte to produce [K_SPECIAL, KS_SPECIAL, KE_FILLER].
            for &b in &buf[..len] {
                self.add_byte(b);
            }
        }
    }
}

/// Save state for redo buffers.
#[derive(Debug, Default)]
pub struct SaveRedo {
    /// Saved redobuff
    pub redobuff: BuffHeader,
    /// Saved old_redobuff
    pub old_redobuff: BuffHeader,
}

impl SaveRedo {
    /// Create a new save state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            redobuff: BuffHeader::new(),
            old_redobuff: BuffHeader::new(),
        }
    }
}

// =============================================================================
// Buffer Access Functions (used by other Rust modules)
// =============================================================================

/// Get a mutable reference to REDOBUFF.
///
/// # Safety
/// Caller must ensure no other mutable references exist.
pub unsafe fn redobuff() -> &'static mut BuffHeader {
    &mut *std::ptr::addr_of_mut!(REDOBUFF)
}

/// Get a mutable reference to OLD_REDOBUFF.
///
/// # Safety
/// Caller must ensure no other mutable references exist.
pub unsafe fn old_redobuff() -> &'static mut BuffHeader {
    &mut *std::ptr::addr_of_mut!(OLD_REDOBUFF)
}

/// Get a mutable reference to RECORDBUFF.
///
/// # Safety
/// Caller must ensure no other mutable references exist.
pub unsafe fn recordbuff() -> &'static mut BuffHeader {
    &mut *std::ptr::addr_of_mut!(RECORDBUFF)
}

/// Get a mutable reference to READBUF1.
///
/// # Safety
/// Caller must ensure no other mutable references exist.
pub unsafe fn readbuf1() -> &'static mut BuffHeader {
    &mut *std::ptr::addr_of_mut!(READBUF1)
}

/// Get a mutable reference to READBUF2.
///
/// # Safety
/// Caller must ensure no other mutable references exist.
pub unsafe fn readbuf2() -> &'static mut BuffHeader {
    &mut *std::ptr::addr_of_mut!(READBUF2)
}

/// Get the block_redo flag.
///
/// # Safety
/// Accesses mutable static.
#[must_use]
pub unsafe fn is_block_redo() -> bool {
    BLOCK_REDO
}

/// Set the block_redo flag.
///
/// # Safety
/// Accesses mutable static.
pub unsafe fn set_block_redo(val: bool) {
    BLOCK_REDO = val;
}

// =============================================================================
// FFI Exports for C callers
// =============================================================================

// --- readbuf1 operations ---

#[no_mangle]
pub unsafe extern "C" fn rs_add_buff_readbuf1(s: *const u8, len: isize) {
    let buf = readbuf1();
    let slice = if len < 0 {
        let mut end = s;
        while *end != 0 {
            end = end.add(1);
        }
        std::slice::from_raw_parts(s, end.offset_from(s) as usize)
    } else {
        std::slice::from_raw_parts(s, len as usize)
    };
    buf.add(slice);
}

#[no_mangle]
pub unsafe extern "C" fn rs_add_char_buff_readbuf1(c: c_int) {
    readbuf1().add_char(c);
}

#[no_mangle]
pub unsafe extern "C" fn rs_add_num_buff_readbuf1(n: c_int) {
    readbuf1().add_num(n);
}

#[no_mangle]
pub unsafe extern "C" fn rs_free_buff_readbuf1() {
    readbuf1().clear();
}

// --- readbuf2 operations ---

#[no_mangle]
pub unsafe extern "C" fn rs_add_buff_readbuf2(s: *const u8, len: isize) {
    let buf = readbuf2();
    let slice = if len < 0 {
        let mut end = s;
        while *end != 0 {
            end = end.add(1);
        }
        std::slice::from_raw_parts(s, end.offset_from(s) as usize)
    } else {
        std::slice::from_raw_parts(s, len as usize)
    };
    buf.add(slice);
}

#[no_mangle]
pub unsafe extern "C" fn rs_add_char_buff_readbuf2(c: c_int) {
    readbuf2().add_char(c);
}

#[no_mangle]
pub unsafe extern "C" fn rs_add_num_buff_readbuf2(n: c_int) {
    readbuf2().add_num(n);
}

#[no_mangle]
pub unsafe extern "C" fn rs_free_buff_readbuf2() {
    readbuf2().clear();
}

// --- redobuff operations ---

#[no_mangle]
pub unsafe extern "C" fn rs_add_buff_redobuff(s: *const u8, len: isize) {
    if BLOCK_REDO {
        return;
    }
    let buf = redobuff();
    let slice = if len < 0 {
        let mut end = s;
        while *end != 0 {
            end = end.add(1);
        }
        std::slice::from_raw_parts(s, end.offset_from(s) as usize)
    } else {
        std::slice::from_raw_parts(s, len as usize)
    };
    buf.add(slice);
}

#[no_mangle]
pub unsafe extern "C" fn rs_add_char_buff_redobuff(c: c_int) {
    if !BLOCK_REDO {
        redobuff().add_char(c);
    }
}

#[no_mangle]
pub unsafe extern "C" fn rs_add_byte_buff_redobuff(c: c_int) {
    if !BLOCK_REDO {
        redobuff().add_byte(c as u8);
    }
}

#[no_mangle]
pub unsafe extern "C" fn rs_add_num_buff_redobuff(n: c_int) {
    if !BLOCK_REDO {
        redobuff().add_num(n);
    }
}

#[no_mangle]
pub unsafe extern "C" fn rs_free_buff_redobuff() {
    redobuff().clear();
}

#[no_mangle]
pub unsafe extern "C" fn rs_get_buffcont_redobuff() -> *mut u8 {
    let contents = redobuff().get_contents();
    if contents.is_empty() {
        return std::ptr::null_mut();
    }
    // Allocate a NUL-terminated C string via xmalloc
    let ptr = xmalloc(contents.len() + 1);
    std::ptr::copy_nonoverlapping(contents.as_ptr(), ptr, contents.len());
    *ptr.add(contents.len()) = 0;
    ptr
}

// --- old_redobuff operations ---

#[no_mangle]
pub unsafe extern "C" fn rs_free_buff_old_redobuff() {
    old_redobuff().clear();
}

// --- recordbuff operations ---

#[no_mangle]
pub unsafe extern "C" fn rs_add_buff_recordbuff(s: *const u8, len: isize) {
    let buf = recordbuff();
    let slice = if len < 0 {
        let mut end = s;
        while *end != 0 {
            end = end.add(1);
        }
        std::slice::from_raw_parts(s, end.offset_from(s) as usize)
    } else {
        std::slice::from_raw_parts(s, len as usize)
    };
    buf.add(slice);
}

#[no_mangle]
pub unsafe extern "C" fn rs_add_char_buff_recordbuff(c: c_int) {
    recordbuff().add_char(c);
}

#[no_mangle]
pub unsafe extern "C" fn rs_add_byte_buff_recordbuff(c: c_int) {
    recordbuff().add_byte(c as u8);
}

#[no_mangle]
pub unsafe extern "C" fn rs_free_buff_recordbuff() {
    recordbuff().clear();
}

#[no_mangle]
pub unsafe extern "C" fn rs_get_buffcont_recordbuff() -> *mut u8 {
    let contents = recordbuff().get_contents();
    if contents.is_empty() {
        return std::ptr::null_mut();
    }
    let ptr = xmalloc(contents.len() + 1);
    std::ptr::copy_nonoverlapping(contents.as_ptr(), ptr, contents.len());
    *ptr.add(contents.len()) = 0;
    ptr
}

#[no_mangle]
pub unsafe extern "C" fn rs_delete_buff_tail_recordbuff(slen: c_int) {
    recordbuff().delete_tail(slen as usize);
}

// --- Cross-buffer operations ---

/// Read from readbuf1 first, fall back to readbuf2.
/// Returns 0 (NUL) if both are empty.
#[no_mangle]
pub unsafe extern "C" fn rs_read_readbuffers(advance: c_int) -> c_int {
    let adv = advance != 0;
    if let Some(c) = readbuf1().read_byte(adv) {
        return c_int::from(c);
    }
    if let Some(c) = readbuf2().read_byte(adv) {
        return c_int::from(c);
    }
    0 // NUL
}

/// Prepare readbufs for reading (start_stuff).
#[no_mangle]
pub unsafe extern "C" fn rs_start_stuff() {
    readbuf1().start_reading();
    readbuf2().start_reading();
}

/// Check if readbuf1 is empty.
#[no_mangle]
pub unsafe extern "C" fn rs_readbuf1_is_empty() -> c_int {
    c_int::from(readbuf1().is_empty())
}

/// Check if readbuf2 is empty.
#[no_mangle]
pub unsafe extern "C" fn rs_readbuf2_is_empty() -> c_int {
    c_int::from(readbuf2().is_empty())
}

/// Get/set block_redo from C.
#[no_mangle]
pub unsafe extern "C" fn rs_get_block_redo() -> c_int {
    c_int::from(BLOCK_REDO)
}

#[no_mangle]
pub unsafe extern "C" fn rs_set_block_redo(val: c_int) {
    BLOCK_REDO = val != 0;
}

// --- Redo buffer swap operations (Phase 2: for C callers) ---

/// ResetRedobuff: swap old_redobuff and redobuff.
/// Does nothing if block_redo is set.
#[no_mangle]
pub unsafe extern "C" fn rs_ResetRedobuff() {
    if BLOCK_REDO {
        return;
    }
    old_redobuff().clear();
    core::ptr::swap(
        std::ptr::addr_of_mut!(REDOBUFF),
        std::ptr::addr_of_mut!(OLD_REDOBUFF),
    );
}

/// CancelRedo: discard redobuff and restore old_redobuff.
/// Drains readbufs. Does nothing if block_redo is set.
#[no_mangle]
pub unsafe extern "C" fn rs_CancelRedo() {
    if BLOCK_REDO {
        return;
    }
    redobuff().clear();
    core::ptr::swap(
        std::ptr::addr_of_mut!(REDOBUFF),
        std::ptr::addr_of_mut!(OLD_REDOBUFF),
    );
    rs_start_stuff();
    while rs_read_readbuffers(1) != 0 {}
}

/// Save redobuff/old_redobuff into opaque save slots.
/// Makes a copy of the saved redobuff back into the active redobuff
/// so that ":normal ." in a function works.
///
/// Uses static save slots (only one level of save supported, matching C).
static mut SAVE_REDOBUFF: BuffHeader = BuffHeader::new();
static mut SAVE_OLD_REDOBUFF: BuffHeader = BuffHeader::new();

#[no_mangle]
pub unsafe extern "C" fn rs_saveRedobuff() {
    // save_redo->sr_redobuff = redobuff; redobuff = empty
    SAVE_REDOBUFF = std::mem::take(&mut *std::ptr::addr_of_mut!(REDOBUFF));
    // save_redo->sr_old_redobuff = old_redobuff; old_redobuff = empty
    SAVE_OLD_REDOBUFF = std::mem::take(&mut *std::ptr::addr_of_mut!(OLD_REDOBUFF));

    // Make a copy so that ":normal ." in a function works.
    let contents = SAVE_REDOBUFF.get_contents();
    if !contents.is_empty() {
        redobuff().add(&contents);
    }
}

#[no_mangle]
pub unsafe extern "C" fn rs_restoreRedobuff() {
    redobuff().clear();
    *std::ptr::addr_of_mut!(REDOBUFF) = std::mem::take(&mut *std::ptr::addr_of_mut!(SAVE_REDOBUFF));
    old_redobuff().clear();
    *std::ptr::addr_of_mut!(OLD_REDOBUFF) =
        std::mem::take(&mut *std::ptr::addr_of_mut!(SAVE_OLD_REDOBUFF));
}

// --- read_redo support ---

/// Static reader state for read_redo.
/// Stores a flattened copy of the buffer content for sequential reading.
static mut REDO_READER_BUF: Vec<u8> = Vec::new();
static mut REDO_READER_POS: usize = 0;

/// Initialize the redo reader. Returns FAIL (1) if nothing to redo, OK (0) otherwise.
#[no_mangle]
pub unsafe extern "C" fn rs_read_redo_init(old_redo: c_int) -> c_int {
    let buf = if old_redo != 0 {
        old_redobuff()
    } else {
        redobuff()
    };
    let contents = buf.get_contents();
    if contents.is_empty() {
        REDO_READER_BUF = Vec::new();
        REDO_READER_POS = 0;
        return 1; // FAIL
    }
    REDO_READER_BUF = contents;
    REDO_READER_POS = 0;
    0 // OK
}

/// Read next byte from redo reader. Returns 0 (NUL) at end.
#[no_mangle]
pub unsafe extern "C" fn rs_read_redo_byte() -> c_int {
    if REDO_READER_POS >= REDO_READER_BUF.len() {
        return 0;
    }
    let c = REDO_READER_BUF[REDO_READER_POS];
    REDO_READER_POS += 1;
    c_int::from(c)
}

/// Peek at the next byte without advancing. Returns 0 at end.
#[no_mangle]
pub unsafe extern "C" fn rs_read_redo_peek() -> c_int {
    if REDO_READER_POS >= REDO_READER_BUF.len() {
        return 0;
    }
    c_int::from(REDO_READER_BUF[REDO_READER_POS])
}

/// Returns 1 (true) if the redo reader has no more bytes to consume.
///
/// Used by rs_copy_redo to distinguish genuine end-of-buffer from a
/// decoded NUL content byte (both cause read_redo_char to return 0).
#[no_mangle]
pub unsafe extern "C" fn rs_redo_reader_at_end() -> c_int {
    c_int::from(REDO_READER_POS >= REDO_READER_BUF.len())
}

// --- Save/restore readbufs for typeahead ---

static mut SAVE_READBUF1: BuffHeader = BuffHeader::new();
static mut SAVE_READBUF2: BuffHeader = BuffHeader::new();

/// Save readbuf1 and readbuf2, clearing the active ones.
#[no_mangle]
pub unsafe extern "C" fn rs_save_readbufs() {
    SAVE_READBUF1 = std::mem::take(&mut *std::ptr::addr_of_mut!(READBUF1));
    SAVE_READBUF2 = std::mem::take(&mut *std::ptr::addr_of_mut!(READBUF2));
}

/// Restore readbuf1 and readbuf2 from saved state.
#[no_mangle]
pub unsafe extern "C" fn rs_restore_readbufs() {
    readbuf1().clear();
    *std::ptr::addr_of_mut!(READBUF1) = std::mem::take(&mut *std::ptr::addr_of_mut!(SAVE_READBUF1));
    readbuf2().clear();
    *std::ptr::addr_of_mut!(READBUF2) = std::mem::take(&mut *std::ptr::addr_of_mut!(SAVE_READBUF2));
}

// --- get_recorded / get_inserted support ---

/// Get recordbuff contents and clear it, then trim last_recorded_len
/// and check for trailing Ctrl_O if restart_edit != 0.
/// Returns xmalloc'd NUL-terminated string, or NULL if empty.
#[no_mangle]
pub unsafe extern "C" fn rs_get_recorded() -> *mut u8 {
    let contents = recordbuff().get_contents();
    recordbuff().clear();

    let last_len = crate::macro_recording::last_recorded_len;
    let restart_edit_val = restart_edit;

    if contents.is_empty() {
        // Match C behavior: get_buffcont with dozero=true returns empty string
        let ptr = xmalloc(1);
        *ptr = 0;
        return ptr;
    }

    let mut len = contents.len();

    // Remove the characters that were added the last time
    if len >= last_len {
        len -= last_len;
    }

    // When stopping recording from Insert mode with CTRL-O q
    const CTRL_O: u8 = 0x0f;
    if len > 0 && restart_edit_val != 0 && contents[len - 1] == CTRL_O {
        len -= 1;
    }

    let ptr = xmalloc(len + 1);
    std::ptr::copy_nonoverlapping(contents.as_ptr(), ptr, len);
    *ptr.add(len) = 0;
    ptr
}

/// Get redobuff contents as xmalloc'd NUL-terminated string.
/// Returns NULL if empty.
#[no_mangle]
pub unsafe extern "C" fn rs_get_inserted() -> *mut u8 {
    let contents = redobuff().get_contents();
    if contents.is_empty() {
        return std::ptr::null_mut();
    }
    let ptr = xmalloc(contents.len() + 1);
    std::ptr::copy_nonoverlapping(contents.as_ptr(), ptr, contents.len());
    *ptr.add(contents.len()) = 0;
    ptr
}

/// Get the length of redobuff contents (for get_inserted).
#[no_mangle]
pub unsafe extern "C" fn rs_get_inserted_len() -> usize {
    redobuff().get_contents().len()
}

extern "C" {
    static mut restart_edit: c_int;
    fn xmalloc(size: usize) -> *mut u8;
}

/// Neovim API `String` type: `{ char *data; size_t size; }`.
///
/// Must match the C layout exactly so it can be returned by value from
/// `#[export_name = "get_inserted"]`.
#[repr(C)]
pub struct NvimString {
    pub data: *mut u8,
    pub size: usize,
}

/// `get_inserted(void)` -- Phase 3 export replacing C wrapper
///
/// Returns the contents of the redo buffer (the last inserted text) as a
/// Neovim API `String` struct.
///
/// # Safety
/// Accesses Rust buffer statics.
#[must_use]
#[export_name = "get_inserted"]
pub unsafe extern "C" fn get_inserted_export() -> NvimString {
    let data = rs_get_inserted();
    let size = rs_get_inserted_len();
    NvimString { data, size }
}

// =============================================================================
// Phase 1: export_name wrappers -- replace C thin wrappers
// =============================================================================

#[allow(
    non_snake_case,
    clippy::module_name_repetitions,
    clippy::wildcard_imports
)]
pub(crate) mod phase1_exports {
    use super::*;

    /// `stuffReadbuff(const char *s)` -- append to stuff buffer (NUL-terminated)
    ///
    /// # Safety
    /// `s` must be a valid NUL-terminated C string pointer.
    #[export_name = "stuffReadbuff"]
    pub unsafe extern "C" fn stuff_readbuff(s: *const u8) {
        rs_add_buff_readbuf1(s, -1);
    }

    /// `stuffReadbuffLen(const char *s, ptrdiff_t len)` -- append to stuff buffer
    ///
    /// # Safety
    /// `s` must be a valid pointer to at least `len` bytes.
    #[export_name = "stuffReadbuffLen"]
    pub unsafe extern "C" fn stuff_readbuff_len(s: *const u8, len: isize) {
        rs_add_buff_readbuf1(s, len);
    }

    /// `stuffRedoReadbuff(const char *s)` -- append to redo stuff buffer
    ///
    /// # Safety
    /// `s` must be a valid NUL-terminated C string pointer.
    #[export_name = "stuffRedoReadbuff"]
    pub unsafe extern "C" fn stuff_redo_readbuff(s: *const u8) {
        rs_add_buff_readbuf2(s, -1);
    }

    /// `stuffcharReadbuff(int c)` -- append char to stuff buffer
    #[export_name = "stuffcharReadbuff"]
    pub unsafe extern "C" fn stuffchar_readbuff(c: c_int) {
        rs_add_char_buff_readbuf1(c);
    }

    /// `stuffnumReadbuff(int n)` -- append number to stuff buffer
    #[export_name = "stuffnumReadbuff"]
    pub unsafe extern "C" fn stuffnum_readbuff(n: c_int) {
        rs_add_num_buff_readbuf1(n);
    }

    /// `AppendToRedobuff(const char *s)` -- append to redo buffer (NUL-terminated)
    ///
    /// # Safety
    /// `s` must be a valid NUL-terminated C string pointer.
    #[export_name = "AppendToRedobuff"]
    pub unsafe extern "C" fn append_to_redobuff(s: *const u8) {
        rs_add_buff_redobuff(s, -1);
    }

    /// `AppendCharToRedobuff(int c)` -- append char to redo buffer
    #[export_name = "AppendCharToRedobuff"]
    pub unsafe extern "C" fn append_char_to_redobuff(c: c_int) {
        rs_add_char_buff_redobuff(c);
    }

    /// `AppendNumberToRedobuff(int n)` -- append number to redo buffer
    #[export_name = "AppendNumberToRedobuff"]
    pub unsafe extern "C" fn append_number_to_redobuff(n: c_int) {
        rs_add_num_buff_redobuff(n);
    }

    /// `AppendToRedobuffLit(const char *str, int len)` -- append literal to redo buffer
    ///
    /// # Safety
    /// `s` must be a valid pointer to at least `len` bytes, or NUL-terminated if len < 0.
    #[export_name = "AppendToRedobuffLit"]
    pub unsafe extern "C" fn append_to_redobuff_lit(s: *const u8, len: c_int) {
        crate::stuff::rs_AppendToRedobuffLit(s, len);
    }

    /// `AppendToRedobuffSpec(const char *s)` -- append with special key escaping to redo buffer
    ///
    /// # Safety
    /// `s` must be a valid NUL-terminated C string pointer.
    #[export_name = "AppendToRedobuffSpec"]
    pub unsafe extern "C" fn append_to_redobuff_spec(s: *const u8) {
        crate::stuff::rs_AppendToRedobuffSpec(s);
    }

    /// `ResetRedobuff(void)` -- reset redo buffer
    #[export_name = "ResetRedobuff"]
    pub unsafe extern "C" fn reset_redobuff() {
        rs_ResetRedobuff();
    }

    /// `CancelRedo(void)` -- cancel redo
    #[export_name = "CancelRedo"]
    pub unsafe extern "C" fn cancel_redo() {
        rs_CancelRedo();
    }

    /// `saveRedobuff(save_redo_T *save_redo)` -- save redo buffers (ignores pointer)
    ///
    /// # Safety
    /// `save_redo` may be any pointer; it is ignored. Save state is in Rust statics.
    #[export_name = "saveRedobuff"]
    pub unsafe extern "C" fn save_redobuff(_save_redo: *mut std::ffi::c_void) {
        rs_saveRedobuff();
    }

    /// `restoreRedobuff(save_redo_T *save_redo)` -- restore redo buffers (ignores pointer)
    ///
    /// # Safety
    /// `save_redo` may be any pointer; it is ignored. Save state is in Rust statics.
    #[export_name = "restoreRedobuff"]
    pub unsafe extern "C" fn restore_redobuff(_save_redo: *mut std::ffi::c_void) {
        rs_restoreRedobuff();
    }

    /// `get_recorded(void)` -- return record buffer contents and clear it
    ///
    /// # Safety
    /// Calls rs_get_recorded().
    #[export_name = "get_recorded"]
    pub unsafe extern "C" fn get_recorded() -> *mut u8 {
        rs_get_recorded()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffblock_new() {
        let block = BuffBlock::new(b"hello");
        assert_eq!(block.content(), b"hello");
        assert_eq!(block.len(), 5);
        assert!(!block.is_empty());
    }

    #[test]
    fn test_buffblock_append() {
        let mut block = BuffBlock::new(b"hello");
        block.append(b" world");
        assert_eq!(block.content(), b"hello world");
    }

    #[test]
    fn test_buffheader_add_and_read() {
        let mut buf = BuffHeader::new();
        assert!(buf.is_empty());

        buf.add(b"hello");
        assert!(!buf.is_empty());

        assert_eq!(buf.read_byte(true), Some(b'h'));
        assert_eq!(buf.read_byte(true), Some(b'e'));
        assert_eq!(buf.read_byte(true), Some(b'l'));
        assert_eq!(buf.read_byte(true), Some(b'l'));
        assert_eq!(buf.read_byte(true), Some(b'o'));
        assert_eq!(buf.read_byte(true), None);
        assert!(buf.is_empty());
    }

    #[test]
    fn test_buffheader_get_contents() {
        let mut buf = BuffHeader::new();
        buf.add(b"hello");
        buf.add(b" world");
        assert_eq!(buf.get_contents(), b"hello world");
    }

    #[test]
    fn test_buffheader_clear() {
        let mut buf = BuffHeader::new();
        buf.add(b"hello");
        buf.clear();
        assert!(buf.is_empty());
    }

    #[test]
    fn test_buffheader_delete_tail() {
        let mut buf = BuffHeader::new();
        buf.add(b"hello");
        buf.delete_tail(2);
        assert_eq!(buf.get_contents(), b"hel");
    }

    // -------------------------------------------------------------------------
    // add_char correctness tests — guard against double-escaping of special keys
    // -------------------------------------------------------------------------

    /// K_PASTE_START = TERMCAP2KEY('P', 'S') = -(0x50 + (0x53 << 8)) = -21328
    /// encode_char must produce [0x80, 0x50, 0x53]; add_char must store it verbatim.
    /// The old double-escape bug would produce [0x80, 0xfe, 0x58, 0x50, 0x53]
    /// (re-encoding the leading 0x80 as K_SPECIAL/KS_SPECIAL/KE_FILLER).
    #[test]
    fn test_add_char_k_paste_start_no_double_escape() {
        const K_PASTE_START: c_int = -(0x50 + (0x53_i32 << 8)); // -21328
        let mut buf = BuffHeader::new();
        buf.add_char(K_PASTE_START);
        // Expected: raw 3-byte special sequence [K_SPECIAL, 'P', 'S']
        assert_eq!(
            buf.get_contents(),
            &[0x80u8, 0x50, 0x53],
            "K_PASTE_START must be stored as [0x80, 'P', 'S'] — not double-escaped"
        );
    }

    /// K_PASTE_END = TERMCAP2KEY('P', 'E') = -(0x50 + (0x45 << 8))
    #[test]
    fn test_add_char_k_paste_end_no_double_escape() {
        const K_PASTE_END: c_int = -(0x50 + (0x45_i32 << 8));
        let mut buf = BuffHeader::new();
        buf.add_char(K_PASTE_END);
        assert_eq!(
            buf.get_contents(),
            &[0x80u8, 0x50, 0x45],
            "K_PASTE_END must be stored as [0x80, 'P', 'E'] — not double-escaped"
        );
    }

    /// NUL (0) must be stored as [K_SPECIAL, KS_ZERO, KE_FILLER] = [0x80, 0xff, 0x58].
    #[test]
    fn test_add_char_nul() {
        let mut buf = BuffHeader::new();
        // KS_ZERO = 255 = 0xff, KE_FILLER = b'X' = 0x58
        buf.add_char(0); // NUL
        assert_eq!(
            buf.get_contents(),
            &[0x80u8, 0xff, 0x58],
            "NUL must be stored as [K_SPECIAL, KS_ZERO, KE_FILLER]"
        );
    }

    /// Literal K_SPECIAL (0x80 as positive c_int) must be stored as
    /// [K_SPECIAL, KS_SPECIAL, KE_FILLER] = [0x80, 0xfe, 0x58].
    #[test]
    fn test_add_char_literal_k_special() {
        let mut buf = BuffHeader::new();
        // KS_SPECIAL = 254 = 0xfe
        buf.add_char(0x80); // K_SPECIAL as positive int
        assert_eq!(
            buf.get_contents(),
            &[0x80u8, 0xfe, 0x58],
            "K_SPECIAL must be stored as [K_SPECIAL, KS_SPECIAL, KE_FILLER]"
        );
    }

    /// Plain ASCII must pass through as-is.
    #[test]
    fn test_add_char_ascii() {
        let mut buf = BuffHeader::new();
        buf.add_char(c_int::from(b'A'));
        assert_eq!(buf.get_contents(), b"A");
    }

    /// U+5000 (倀) encodes as UTF-8 [0xE5, 0x80, 0x80].  The two 0x80 bytes are
    /// content bytes that must be escaped by add_byte (matching C add_char_buff):
    /// each 0x80 content byte becomes [K_SPECIAL=0x80, KS_SPECIAL=0xFE, KE_FILLER=0x58].
    #[test]
    fn test_add_char_multibyte_with_0x80_bytes() {
        let mut buf = BuffHeader::new();
        // UTF-8 of U+5000 is [E5, 80, 80].
        // 0xE5 is stored raw; each 0x80 content byte is escaped via add_byte.
        buf.add_char(0x5000); // U+5000 倀
        assert_eq!(
            buf.get_contents(),
            // 0xE5 raw, then 0x80 -> [0x80, 0xFE, 0x58], then 0x80 -> [0x80, 0xFE, 0x58]
            &[0xE5u8, 0x80, 0xFE, 0x58, 0x80, 0xFE, 0x58],
            "Content 0x80 bytes must be escaped as [K_SPECIAL, KS_SPECIAL, KE_FILLER]"
        );
    }

    /// Round-trip test: two K_PASTE_START chars stored via add_char then read
    /// back byte-by-byte must reproduce the two 3-byte sequences.
    #[test]
    fn test_add_char_k_paste_start_round_trip() {
        const K_PASTE_START: c_int = -(0x50 + (0x53_i32 << 8));
        let mut buf = BuffHeader::new();
        buf.add_char(K_PASTE_START);
        buf.add_char(K_PASTE_START);
        let expected: &[u8] = &[0x80, 0x50, 0x53, 0x80, 0x50, 0x53];
        assert_eq!(buf.get_contents(), expected);
    }
}
