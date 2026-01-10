//! Word tree compression utilities for spell files.
//!
//! This module provides utilities for compressing word trees during
//! spell file generation. The compression works by finding identical
//! subtrees and merging them to reduce file size.

#![allow(clippy::cast_lossless)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::option_if_let_else)]

use std::ffi::c_int;

// =============================================================================
// Compression Statistics
// =============================================================================

/// Statistics from word tree compression.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct CompressionStats {
    /// Total number of nodes before compression
    pub nodes_before: u32,
    /// Total number of nodes after compression
    pub nodes_after: u32,
    /// Number of nodes that were merged
    pub nodes_merged: u32,
    /// Number of unique subtrees found
    pub unique_subtrees: u32,
    /// Maximum tree depth
    pub max_depth: u16,
    /// Number of compression passes performed
    pub passes: u16,
}

impl CompressionStats {
    /// Create new compression statistics.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            nodes_before: 0,
            nodes_after: 0,
            nodes_merged: 0,
            unique_subtrees: 0,
            max_depth: 0,
            passes: 0,
        }
    }

    /// Calculate the compression ratio.
    ///
    /// Returns the ratio of nodes after to nodes before, as a percentage.
    /// Returns 100 if there were no nodes before.
    #[must_use]
    pub const fn compression_ratio(&self) -> u32 {
        if self.nodes_before == 0 {
            return 100;
        }
        (self.nodes_after * 100) / self.nodes_before
    }

    /// Calculate the space saved in percentage.
    #[must_use]
    pub const fn space_saved_percent(&self) -> u32 {
        100 - self.compression_ratio()
    }

    /// Calculate number of nodes saved.
    #[must_use]
    pub const fn nodes_saved(&self) -> u32 {
        self.nodes_before.saturating_sub(self.nodes_after)
    }
}

// =============================================================================
// Compression Thresholds
// =============================================================================

/// Configuration for when to trigger compression during word loading.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CompressionConfig {
    /// Memory threshold to start compression (in units of SBLOCKSIZE)
    pub memory_start: u32,
    /// Memory increment between compression passes
    pub memory_increment: u32,
    /// Word count threshold for additional compression
    pub word_count_threshold: u32,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl CompressionConfig {
    /// Create default compression configuration.
    ///
    /// These values match the defaults in spellfile.c:
    /// - compress_start = 30000
    /// - compress_inc = 100
    /// - compress_added = 500000
    #[must_use]
    pub const fn new() -> Self {
        Self {
            memory_start: 30000,
            memory_increment: 100,
            word_count_threshold: 500_000,
        }
    }

    /// Check if compression should be triggered based on memory usage.
    ///
    /// # Arguments
    /// * `memory_blocks` - Current memory usage in blocks
    /// * `compress_count` - Number of compression passes already done
    #[must_use]
    pub const fn should_compress_memory(&self, memory_blocks: u32, compress_count: u32) -> bool {
        memory_blocks >= self.memory_start + compress_count * self.memory_increment
    }

    /// Check if compression should be triggered based on word count.
    #[must_use]
    pub const fn should_compress_words(&self, word_count: u32) -> bool {
        word_count >= self.word_count_threshold
    }
}

// =============================================================================
// Node Hash Computation
// =============================================================================

/// Hash value for node comparison during compression.
pub type NodeHash = u64;

/// Compute hash for node comparison.
///
/// This is used to quickly identify potentially identical subtrees.
/// Two nodes with different hashes are definitely different;
/// two nodes with the same hash might be identical (need full comparison).
#[must_use]
pub const fn compute_node_hash(byte_val: u8, flags: u8, sibling_count: u16) -> NodeHash {
    let mut hash: NodeHash = byte_val as NodeHash;
    hash = hash.wrapping_mul(31).wrapping_add(flags as NodeHash);
    hash = hash
        .wrapping_mul(31)
        .wrapping_add(sibling_count as NodeHash);
    hash
}

/// Combine parent hash with child hash.
#[must_use]
pub const fn combine_hashes(parent: NodeHash, child: NodeHash) -> NodeHash {
    parent.wrapping_mul(31).wrapping_add(child)
}

// =============================================================================
// Compression Pass State
// =============================================================================

/// State maintained during a compression pass.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct CompressionPass {
    /// Current node being processed
    pub current_node: u32,
    /// Total nodes processed in this pass
    pub nodes_processed: u32,
    /// Nodes compressed in this pass
    pub nodes_compressed: u32,
    /// Whether any compression happened
    pub changed: bool,
}

impl CompressionPass {
    /// Create new compression pass state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            current_node: 0,
            nodes_processed: 0,
            nodes_compressed: 0,
            changed: false,
        }
    }

    /// Reset for a new pass.
    pub fn reset(&mut self) {
        self.current_node = 0;
        self.nodes_processed = 0;
        self.nodes_compressed = 0;
        self.changed = false;
    }

    /// Record that a node was compressed.
    pub fn record_compression(&mut self) {
        self.nodes_compressed += 1;
        self.changed = true;
    }

    /// Advance to next node.
    pub fn advance(&mut self) {
        self.current_node += 1;
        self.nodes_processed += 1;
    }
}

// =============================================================================
// Index Offset Utilities
// =============================================================================

/// Maximum value that fits in a 3-byte offset (24 bits).
pub const MAX_OFFSET_3BYTE: u32 = 0x00FF_FFFF;

/// Maximum value that fits in a 4-byte offset (32 bits).
pub const MAX_OFFSET_4BYTE: u32 = u32::MAX;

/// Calculate the number of bytes needed to store an index offset.
#[must_use]
pub const fn offset_byte_count(offset: u32) -> usize {
    if offset <= MAX_OFFSET_3BYTE {
        3
    } else {
        4
    }
}

/// Encode offset to bytes (big-endian, variable length).
///
/// Returns the number of bytes written.
#[must_use]
pub fn encode_offset(buf: &mut [u8], offset: u32) -> Option<usize> {
    let byte_count = offset_byte_count(offset);
    if buf.len() < byte_count {
        return None;
    }

    match byte_count {
        3 => {
            buf[0] = ((offset >> 16) & 0xFF) as u8;
            buf[1] = ((offset >> 8) & 0xFF) as u8;
            buf[2] = (offset & 0xFF) as u8;
        }
        4 => {
            buf[0] = ((offset >> 24) & 0xFF) as u8;
            buf[1] = ((offset >> 16) & 0xFF) as u8;
            buf[2] = ((offset >> 8) & 0xFF) as u8;
            buf[3] = (offset & 0xFF) as u8;
        }
        _ => return None,
    }

    Some(byte_count)
}

/// Decode offset from bytes (big-endian, variable length).
#[must_use]
pub fn decode_offset_3byte(buf: &[u8]) -> Option<u32> {
    if buf.len() < 3 {
        return None;
    }
    Some(((buf[0] as u32) << 16) | ((buf[1] as u32) << 8) | (buf[2] as u32))
}

/// Decode 4-byte offset from bytes (big-endian).
#[must_use]
pub fn decode_offset_4byte(buf: &[u8]) -> Option<u32> {
    if buf.len() < 4 {
        return None;
    }
    Some(
        ((buf[0] as u32) << 24)
            | ((buf[1] as u32) << 16)
            | ((buf[2] as u32) << 8)
            | (buf[3] as u32),
    )
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Create new compression statistics.
#[no_mangle]
pub extern "C" fn rs_compression_stats_new() -> CompressionStats {
    CompressionStats::new()
}

/// Get compression ratio as percentage.
///
/// # Safety
/// `stats` must be a valid pointer if non-null.
#[no_mangle]
pub unsafe extern "C" fn rs_compression_stats_ratio(stats: *const CompressionStats) -> c_int {
    if stats.is_null() {
        return 100;
    }
    (*stats).compression_ratio() as c_int
}

/// Get space saved as percentage.
///
/// # Safety
/// `stats` must be a valid pointer if non-null.
#[no_mangle]
pub unsafe extern "C" fn rs_compression_stats_saved(stats: *const CompressionStats) -> c_int {
    if stats.is_null() {
        return 0;
    }
    (*stats).space_saved_percent() as c_int
}

/// Create default compression config.
#[no_mangle]
pub extern "C" fn rs_compression_config_new() -> CompressionConfig {
    CompressionConfig::new()
}

/// Check if compression should trigger based on memory.
///
/// # Safety
/// `config` must be a valid pointer if non-null.
#[no_mangle]
pub unsafe extern "C" fn rs_should_compress_memory(
    config: *const CompressionConfig,
    memory_blocks: u32,
    compress_count: u32,
) -> c_int {
    if config.is_null() {
        return 0;
    }
    c_int::from((*config).should_compress_memory(memory_blocks, compress_count))
}

/// Check if compression should trigger based on word count.
///
/// # Safety
/// `config` must be a valid pointer if non-null.
#[no_mangle]
pub unsafe extern "C" fn rs_should_compress_words(
    config: *const CompressionConfig,
    word_count: u32,
) -> c_int {
    if config.is_null() {
        return 0;
    }
    c_int::from((*config).should_compress_words(word_count))
}

/// Compute node hash.
#[no_mangle]
pub extern "C" fn rs_compute_node_hash(byte_val: u8, flags: u8, sibling_count: u16) -> NodeHash {
    compute_node_hash(byte_val, flags, sibling_count)
}

/// Combine hashes.
#[no_mangle]
pub extern "C" fn rs_combine_hashes(parent: NodeHash, child: NodeHash) -> NodeHash {
    combine_hashes(parent, child)
}

/// Get bytes needed for offset.
#[no_mangle]
pub extern "C" fn rs_offset_byte_count(offset: u32) -> c_int {
    offset_byte_count(offset) as c_int
}

/// Encode offset to buffer.
///
/// # Safety
/// `buf` must point to a valid buffer of at least 4 bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_encode_offset(
    buf: *mut u8,
    buf_len: usize,
    offset: u32,
    written: *mut usize,
) -> c_int {
    if buf.is_null() || written.is_null() {
        return -1;
    }

    let slice = std::slice::from_raw_parts_mut(buf, buf_len);
    match encode_offset(slice, offset) {
        Some(n) => {
            *written = n;
            0
        }
        None => -1,
    }
}

/// Decode 3-byte offset from buffer.
///
/// # Safety
/// `buf` must point to a valid buffer of at least 3 bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_decode_offset_3byte(
    buf: *const u8,
    buf_len: usize,
    result: *mut u32,
) -> c_int {
    if buf.is_null() || result.is_null() {
        return -1;
    }

    let slice = std::slice::from_raw_parts(buf, buf_len);
    match decode_offset_3byte(slice) {
        Some(v) => {
            *result = v;
            0
        }
        None => -1,
    }
}

/// Decode 4-byte offset from buffer.
///
/// # Safety
/// `buf` must point to a valid buffer of at least 4 bytes.
#[no_mangle]
pub unsafe extern "C" fn rs_decode_offset_4byte(
    buf: *const u8,
    buf_len: usize,
    result: *mut u32,
) -> c_int {
    if buf.is_null() || result.is_null() {
        return -1;
    }

    let slice = std::slice::from_raw_parts(buf, buf_len);
    match decode_offset_4byte(slice) {
        Some(v) => {
            *result = v;
            0
        }
        None => -1,
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_stats() {
        let mut stats = CompressionStats::new();
        assert_eq!(stats.compression_ratio(), 100);
        assert_eq!(stats.space_saved_percent(), 0);
        assert_eq!(stats.nodes_saved(), 0);

        stats.nodes_before = 1000;
        stats.nodes_after = 750;
        assert_eq!(stats.compression_ratio(), 75);
        assert_eq!(stats.space_saved_percent(), 25);
        assert_eq!(stats.nodes_saved(), 250);
    }

    #[test]
    fn test_compression_config() {
        let config = CompressionConfig::new();
        assert_eq!(config.memory_start, 30000);
        assert_eq!(config.memory_increment, 100);
        assert_eq!(config.word_count_threshold, 500_000);

        // First compression at memory_start
        assert!(!config.should_compress_memory(29999, 0));
        assert!(config.should_compress_memory(30000, 0));
        assert!(config.should_compress_memory(30001, 0));

        // Second compression at memory_start + increment
        assert!(!config.should_compress_memory(30099, 1));
        assert!(config.should_compress_memory(30100, 1));

        // Word count threshold
        assert!(!config.should_compress_words(499_999));
        assert!(config.should_compress_words(500_000));
    }

    #[test]
    fn test_node_hash() {
        let hash1 = compute_node_hash(b'a', 0, 0);
        let hash2 = compute_node_hash(b'a', 0, 0);
        let hash3 = compute_node_hash(b'b', 0, 0);

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);

        let combined = combine_hashes(hash1, hash3);
        assert_ne!(combined, hash1);
        assert_ne!(combined, hash3);
    }

    #[test]
    fn test_compression_pass() {
        let mut pass = CompressionPass::new();
        assert_eq!(pass.current_node, 0);
        assert!(!pass.changed);

        pass.advance();
        assert_eq!(pass.current_node, 1);
        assert_eq!(pass.nodes_processed, 1);

        pass.record_compression();
        assert_eq!(pass.nodes_compressed, 1);
        assert!(pass.changed);

        pass.reset();
        assert_eq!(pass.current_node, 0);
        assert!(!pass.changed);
    }

    #[test]
    fn test_offset_byte_count() {
        assert_eq!(offset_byte_count(0), 3);
        assert_eq!(offset_byte_count(MAX_OFFSET_3BYTE), 3);
        assert_eq!(offset_byte_count(MAX_OFFSET_3BYTE + 1), 4);
        assert_eq!(offset_byte_count(MAX_OFFSET_4BYTE), 4);
    }

    #[test]
    fn test_encode_decode_offset_3byte() {
        let mut buf = [0u8; 4];

        let written = encode_offset(&mut buf, 0x0012_3456).unwrap();
        assert_eq!(written, 3);
        assert_eq!(buf[0], 0x12);
        assert_eq!(buf[1], 0x34);
        assert_eq!(buf[2], 0x56);

        let decoded = decode_offset_3byte(&buf).unwrap();
        assert_eq!(decoded, 0x0012_3456);
    }

    #[test]
    fn test_encode_decode_offset_4byte() {
        let mut buf = [0u8; 4];

        let written = encode_offset(&mut buf, 0x1234_5678).unwrap();
        assert_eq!(written, 4);
        assert_eq!(buf[0], 0x12);
        assert_eq!(buf[1], 0x34);
        assert_eq!(buf[2], 0x56);
        assert_eq!(buf[3], 0x78);

        let decoded = decode_offset_4byte(&buf).unwrap();
        assert_eq!(decoded, 0x1234_5678);
    }

    #[test]
    fn test_encode_offset_buffer_too_small() {
        let mut buf = [0u8; 2];
        assert!(encode_offset(&mut buf, 0x0012_3456).is_none());
    }
}
