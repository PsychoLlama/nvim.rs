//! Word tree node helpers for spell file construction
//!
//! This module provides helpers for building and manipulating word trees
//! used in spell checking. Word trees are compact tries stored as parallel
//! byte/index arrays.

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::similar_names)]
#![allow(clippy::unreadable_literal)]

use std::ffi::c_int;

use crate::wordtree::{WF_ALLCAP, WF_BANNED, WF_KEEPCAP, WF_ONECAP, WF_RARE, WF_REGION};
use crate::IdxT;

// =============================================================================
// Word Node Constants
// =============================================================================

/// Maximum number of siblings at a node
pub const MAX_SIBLINGS: usize = 256;

/// Byte value indicating end of word
pub const NUL_BYTE: u8 = 0;

/// Index value indicating no child
pub const NO_INDEX: IdxT = 0;

// =============================================================================
// Word Node Flags
// =============================================================================

/// Flags stored with word endings
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct WordFlags {
    /// Raw flags value
    pub flags: u32,
    /// Region mask (which regions this word is valid in)
    pub region: u8,
    /// Affix ID (for prefix/suffix rules)
    pub affix_id: u16,
}

impl WordFlags {
    /// Create empty flags.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            flags: 0,
            region: 0,
            affix_id: 0,
        }
    }

    /// Create flags from a raw value.
    #[must_use]
    pub const fn from_raw(raw: u32) -> Self {
        Self {
            flags: raw & 0xFFFF,
            region: ((raw >> 16) & 0xFF) as u8,
            affix_id: 0,
        }
    }

    /// Convert to raw value for storage.
    #[must_use]
    pub const fn to_raw(&self) -> u32 {
        self.flags | ((self.region as u32) << 16)
    }

    /// Check if word has region info.
    #[must_use]
    pub const fn has_region(&self) -> bool {
        (self.flags & WF_REGION) != 0
    }

    /// Check if word has ONECAP flag.
    #[must_use]
    pub const fn is_onecap(&self) -> bool {
        (self.flags & WF_ONECAP) != 0
    }

    /// Check if word has ALLCAP flag.
    #[must_use]
    pub const fn is_allcap(&self) -> bool {
        (self.flags & WF_ALLCAP) != 0
    }

    /// Check if word has KEEPCAP flag.
    #[must_use]
    pub const fn is_keepcap(&self) -> bool {
        (self.flags & WF_KEEPCAP) != 0
    }

    /// Check if word is rare.
    #[must_use]
    pub const fn is_rare(&self) -> bool {
        (self.flags & WF_RARE) != 0
    }

    /// Check if word is banned (bad word).
    #[must_use]
    pub const fn is_banned(&self) -> bool {
        (self.flags & WF_BANNED) != 0
    }

    /// Set ONECAP flag.
    pub fn set_onecap(&mut self) {
        self.flags |= WF_ONECAP;
    }

    /// Set ALLCAP flag.
    pub fn set_allcap(&mut self) {
        self.flags |= WF_ALLCAP;
    }

    /// Set KEEPCAP flag.
    pub fn set_keepcap(&mut self) {
        self.flags |= WF_KEEPCAP;
    }

    /// Set rare flag.
    pub fn set_rare(&mut self) {
        self.flags |= WF_RARE;
    }

    /// Set banned flag.
    pub fn set_banned(&mut self) {
        self.flags |= WF_BANNED;
    }

    /// Set region mask.
    pub fn set_region(&mut self, region: u8) {
        self.region = region;
        if region != 0 {
            self.flags |= WF_REGION;
        }
    }

    /// Check if word is valid in a specific region.
    #[must_use]
    pub const fn valid_in_region(&self, region_mask: u8) -> bool {
        // If no region specified, valid everywhere
        if !self.has_region() {
            return true;
        }
        // Check if any requested region is valid
        (self.region & region_mask) != 0
    }
}

// =============================================================================
// Word Node Helpers
// =============================================================================

/// Information about a node in the word tree.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct NodeInfo {
    /// Index of this node in the byts/idxs arrays
    pub idx: IdxT,
    /// Number of siblings (including this byte)
    pub sibling_count: u8,
    /// This node's byte value
    pub byte_val: u8,
    /// Whether this is a word end (byte_val == 0)
    pub is_word_end: bool,
}

/// Parse node info from byte/index arrays.
///
/// # Arguments
///
/// * `byts` - Bytes array
/// * `idxs` - Index array
/// * `idx` - Index to read from
///
/// # Returns
///
/// Node info and the index of the first sibling
#[must_use]
pub fn parse_node_info(byts: &[u8], idxs: &[IdxT], idx: usize) -> Option<(NodeInfo, usize)> {
    if idx >= byts.len() || idx >= idxs.len() {
        return None;
    }

    let sibling_count = byts[idx];
    if sibling_count == 0 {
        return None;
    }

    Some((
        NodeInfo {
            idx: idx as IdxT,
            sibling_count,
            byte_val: 0,
            is_word_end: false,
        },
        idx + 1,
    ))
}

/// Get the byte value and child index for a sibling.
///
/// # Arguments
///
/// * `byts` - Bytes array
/// * `idxs` - Index array
/// * `first_sibling` - Index of first sibling byte
/// * `sibling_offset` - Offset from first sibling (0 to count-1)
///
/// # Returns
///
/// (byte value, child index or flags)
#[must_use]
pub fn get_sibling(
    byts: &[u8],
    idxs: &[IdxT],
    first_sibling: usize,
    sibling_offset: usize,
) -> Option<(u8, IdxT)> {
    let idx = first_sibling + sibling_offset;
    if idx >= byts.len() || idx >= idxs.len() {
        return None;
    }

    Some((byts[idx], idxs[idx]))
}

/// Count word endings at a node.
///
/// Word endings are indicated by consecutive NUL bytes at the start
/// of the sibling list.
///
/// # Arguments
///
/// * `byts` - Bytes array
/// * `first_sibling` - Index of first sibling byte
/// * `count` - Number of siblings
///
/// # Returns
///
/// Number of word endings
#[must_use]
pub fn count_word_endings(byts: &[u8], first_sibling: usize, count: usize) -> usize {
    let mut endings = 0;
    for i in 0..count {
        let idx = first_sibling + i;
        if idx >= byts.len() {
            break;
        }
        if byts[idx] == NUL_BYTE {
            endings += 1;
        } else {
            // Siblings are sorted, NULs come first
            break;
        }
    }
    endings
}

/// Find a byte in the sibling list using binary search.
///
/// # Arguments
///
/// * `byts` - Bytes array
/// * `first_sibling` - Index of first sibling byte
/// * `count` - Number of siblings
/// * `byte` - Byte to find
///
/// # Returns
///
/// Index of the byte if found, or None
#[must_use]
pub fn find_sibling_byte(
    byts: &[u8],
    first_sibling: usize,
    count: usize,
    byte: u8,
) -> Option<usize> {
    if count == 0 {
        return None;
    }

    // Skip word ending NULs (they come first)
    let mut lo = 0;
    while lo < count && first_sibling + lo < byts.len() && byts[first_sibling + lo] == NUL_BYTE {
        lo += 1;
    }

    if byte == NUL_BYTE {
        // Looking for word end - if there are NULs, the first one is at index 0
        return if lo > 0 { Some(first_sibling) } else { None };
    }

    // Binary search in the non-NUL portion
    let mut hi = count - 1;
    while lo <= hi {
        let mid = lo + (hi - lo) / 2;
        let idx = first_sibling + mid;
        if idx >= byts.len() {
            return None;
        }

        let mid_byte = byts[idx];
        match mid_byte.cmp(&byte) {
            std::cmp::Ordering::Equal => return Some(idx),
            std::cmp::Ordering::Less => lo = mid + 1,
            std::cmp::Ordering::Greater => {
                if mid == 0 {
                    return None;
                }
                hi = mid - 1;
            }
        }
    }

    None
}

// =============================================================================
// Tree Statistics
// =============================================================================

/// Statistics about a word tree.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct TreeStats {
    /// Total number of nodes
    pub node_count: u32,
    /// Total number of word endings
    pub word_count: u32,
    /// Maximum depth of tree
    pub max_depth: u32,
    /// Total bytes used
    pub total_bytes: u32,
}

/// Calculate tree statistics.
///
/// # Arguments
///
/// * `byts` - Bytes array
/// * `idxs` - Index array
/// * `root_idx` - Root node index
///
/// # Returns
///
/// Tree statistics
#[must_use]
pub fn calc_tree_stats(byts: &[u8], idxs: &[IdxT], root_idx: usize) -> TreeStats {
    let mut stats = TreeStats {
        total_bytes: byts.len() as u32,
        ..Default::default()
    };

    if root_idx >= byts.len() {
        return stats;
    }

    // Simple traversal to count
    calc_stats_recursive(byts, idxs, root_idx, 0, &mut stats);

    stats
}

fn calc_stats_recursive(byts: &[u8], idxs: &[IdxT], idx: usize, depth: u32, stats: &mut TreeStats) {
    if idx >= byts.len() {
        return;
    }

    stats.node_count += 1;
    if depth > stats.max_depth {
        stats.max_depth = depth;
    }

    let count = byts[idx] as usize;
    if count == 0 {
        return;
    }

    let first_sibling = idx + 1;

    for i in 0..count {
        let sibling_idx = first_sibling + i;
        if sibling_idx >= byts.len() || sibling_idx >= idxs.len() {
            break;
        }

        let byte_val = byts[sibling_idx];
        let child_idx = idxs[sibling_idx] as usize;

        if byte_val == NUL_BYTE {
            // Word ending
            stats.word_count += 1;
        } else if child_idx > 0 && child_idx < byts.len() {
            // Recurse into child
            calc_stats_recursive(byts, idxs, child_idx, depth + 1, stats);
        }
    }
}

// =============================================================================
// FFI Exports
// =============================================================================

/// Create word flags from raw value.
#[unsafe(no_mangle)]
pub extern "C" fn rs_word_flags_from_raw(raw: u32) -> WordFlags {
    WordFlags::from_raw(raw)
}

/// Convert word flags to raw value.
#[unsafe(no_mangle)]
pub extern "C" fn rs_word_flags_to_raw(flags: &WordFlags) -> u32 {
    flags.to_raw()
}

/// Check if word flags indicate ONECAP.
#[unsafe(no_mangle)]
pub extern "C" fn rs_word_flags_is_onecap(flags: u32) -> c_int {
    c_int::from((flags & WF_ONECAP) != 0)
}

/// Check if word flags indicate ALLCAP.
#[unsafe(no_mangle)]
pub extern "C" fn rs_word_flags_is_allcap(flags: u32) -> c_int {
    c_int::from((flags & WF_ALLCAP) != 0)
}

/// Check if word flags indicate KEEPCAP.
#[unsafe(no_mangle)]
pub extern "C" fn rs_word_flags_is_keepcap(flags: u32) -> c_int {
    c_int::from((flags & WF_KEEPCAP) != 0)
}

/// Check if word flags indicate rare.
#[unsafe(no_mangle)]
pub extern "C" fn rs_word_flags_is_rare(flags: u32) -> c_int {
    c_int::from((flags & WF_RARE) != 0)
}

/// Check if word flags indicate banned.
#[unsafe(no_mangle)]
pub extern "C" fn rs_word_flags_is_banned(flags: u32) -> c_int {
    c_int::from((flags & WF_BANNED) != 0)
}

/// Check if word is valid in region.
#[unsafe(no_mangle)]
pub extern "C" fn rs_word_valid_in_region(flags: u32, region_mask: u8) -> c_int {
    let wf = WordFlags::from_raw(flags);
    c_int::from(wf.valid_in_region(region_mask))
}

/// Count word endings at a node.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_count_word_endings(
    byts: *const u8,
    byts_len: usize,
    first_sibling: usize,
    count: usize,
) -> c_int {
    if byts.is_null() {
        return 0;
    }
    let slice = std::slice::from_raw_parts(byts, byts_len);
    count_word_endings(slice, first_sibling, count) as c_int
}

/// Find a byte in sibling list.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_find_sibling_byte(
    byts: *const u8,
    byts_len: usize,
    first_sibling: usize,
    count: usize,
    byte: u8,
) -> c_int {
    if byts.is_null() {
        return -1;
    }
    let slice = std::slice::from_raw_parts(byts, byts_len);
    find_sibling_byte(slice, first_sibling, count, byte).map_or(-1, |idx| idx as c_int)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_flags() {
        let mut flags = WordFlags::empty();
        assert!(!flags.is_onecap());
        assert!(!flags.is_rare());

        flags.set_onecap();
        assert!(flags.is_onecap());
        assert!(!flags.is_allcap());

        flags.set_rare();
        assert!(flags.is_rare());

        flags.set_region(0b00000011); // Regions 1 and 2
        assert!(flags.has_region());
        assert!(flags.valid_in_region(0b00000001)); // Region 1
        assert!(flags.valid_in_region(0b00000010)); // Region 2
        assert!(!flags.valid_in_region(0b00000100)); // Region 3
    }

    #[test]
    fn test_word_flags_raw_conversion() {
        let mut flags = WordFlags::empty();
        flags.flags = 0x0F;
        flags.region = 0x55;

        let raw = flags.to_raw();
        let restored = WordFlags::from_raw(raw);

        assert_eq!(restored.flags, 0x0F);
        assert_eq!(restored.region, 0x55);
    }

    #[test]
    fn test_count_word_endings() {
        // Sibling list: [NUL, NUL, 'a', 'b']
        let byts = [4u8, 0, 0, b'a', b'b'];
        assert_eq!(count_word_endings(&byts, 1, 4), 2);

        // No NULs
        let byts2 = [2u8, b'a', b'b'];
        assert_eq!(count_word_endings(&byts2, 1, 2), 0);
    }

    #[test]
    fn test_find_sibling_byte() {
        // Sibling list at index 1: [NUL, NUL, 'a', 'c', 'z']
        let byts = [5u8, 0, 0, b'a', b'c', b'z'];

        // Find NUL
        assert_eq!(find_sibling_byte(&byts, 1, 5, 0), Some(1));

        // Find 'a'
        assert_eq!(find_sibling_byte(&byts, 1, 5, b'a'), Some(3));

        // Find 'c'
        assert_eq!(find_sibling_byte(&byts, 1, 5, b'c'), Some(4));

        // Find 'z'
        assert_eq!(find_sibling_byte(&byts, 1, 5, b'z'), Some(5));

        // Not found
        assert_eq!(find_sibling_byte(&byts, 1, 5, b'b'), None);
        assert_eq!(find_sibling_byte(&byts, 1, 5, b'x'), None);
    }
}
