//! Buffer header implementation for stuff/redo/recording buffers
//!
//! This module provides safe Rust wrappers for the `buffheader_T` and
//! `buffblock_T` types used for managing the stuff buffer, redo buffer,
//! and recording buffer.

/// Minimum size for a buffer block's string content.
const MINIMAL_SIZE: usize = 20;

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
    /// Translates special keys, NUL, and K_SPECIAL.
    pub fn add_byte(&mut self, c: u8) {
        const K_SPECIAL: u8 = 0x80;
        const NUL: u8 = 0;

        if c == K_SPECIAL || c == NUL {
            // Need to escape: K_SPECIAL + second + third byte
            // For K_SPECIAL: K_SPECIAL KS_SPECIAL KE_FILLER
            // For NUL: K_SPECIAL KS_ZERO KE_FILLER
            let ks = if c == K_SPECIAL { 254 } else { 255 }; // KS_SPECIAL or KS_ZERO
            let ke = b'X'; // KE_FILLER
            self.add(&[K_SPECIAL, ks, ke]);
        } else {
            self.add(&[c]);
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
}
