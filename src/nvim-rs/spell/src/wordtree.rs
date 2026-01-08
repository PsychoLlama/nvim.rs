//! Word tree traversal for Neovim spell checking
//!
//! This module provides functions for traversing Neovim's spell word trees.
//! The trees use a compact trie structure stored in parallel byts/idxs arrays.
//!
//! Tree structure:
//! - `byts[idx]` contains the sibling count at a node
//! - `byts[idx+1..idx+1+count]` contains the sibling bytes (sorted)
//! - `idxs[idx+1..idx+1+count]` contains either:
//!   - For byte 0: flags/region (word end marker)
//!   - For other bytes: child node index
//!
//! A byte of 0 indicates a word end. Multiple 0s in a row indicate
//! the same word with different flags/regions.

#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_sign_loss)]

use std::ffi::c_int;

use crate::IdxT;

/// Maximum word length in bytes
pub const MAXWLEN: usize = 254;

// =============================================================================
// Word Flags from spell_defs.h
// =============================================================================

/// Word has region byte
pub const WF_REGION: u32 = 0x01;
/// Word has ONECAP (first char uppercase)
pub const WF_ONECAP: u32 = 0x02;
/// Word has ALLCAP (all chars uppercase)
pub const WF_ALLCAP: u32 = 0x04;
/// Word is rare
pub const WF_RARE: u32 = 0x08;
/// Word is banned (bad word)
pub const WF_BANNED: u32 = 0x10;
/// Word has affix ID
pub const WF_AFX: u32 = 0x20;
/// Word has FIXCAP (case must match exactly)
pub const WF_FIXCAP: u32 = 0x40;
/// Word has KEEPCAP (preserve case from dictionary)
pub const WF_KEEPCAP: u32 = 0x80;
/// Word requires compounding
pub const WF_NEEDCOMP: u32 = 0x100;
/// Word not allowed at start of compound
pub const WF_NOCOMPBEF: u32 = 0x200;
/// Word not allowed at end of compound
pub const WF_NOCOMPAFT: u32 = 0x400;
/// Word is compound root
pub const WF_COMPROOT: u32 = 0x800;
/// Prefix has rare flag
pub const WF_RAREPFX: u32 = 0x1000;

/// Mask for compound flag byte (bits 24-31)
pub const WF_COMPFLAG_MASK: u32 = 0xFF00_0000;

/// Extracts the compound flag from word flags
#[must_use]
pub const fn get_compound_flag(flags: u32) -> u8 {
    ((flags >> 24) & 0xFF) as u8
}

// =============================================================================
// Match Result Types
// =============================================================================

/// Result of a tree search operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeSearchResult {
    /// Word found with the given flags
    Found {
        /// Length of the matched word in bytes
        word_len: usize,
        /// Array index of the word end marker (for accessing flags)
        end_idx: usize,
    },
    /// Multiple possible word endings found
    MultipleEndings {
        /// Pairs of (word_len, end_idx) for each possible ending
        endings: Vec<(usize, usize)>,
    },
    /// Word not found in tree
    NotFound,
    /// Tree is empty or invalid
    Empty,
}

/// Performs a binary search in a sorted sibling list within the tree.
///
/// Given a range of indices in the byts array (a sorted list of sibling bytes),
/// searches for the given byte value.
///
/// # Arguments
/// * `byts` - The bytes array of the tree
/// * `lo` - Start index of the search range
/// * `hi` - End index of the search range (inclusive)
/// * `c` - The byte to search for
///
/// # Returns
/// * `Some(idx)` if the byte is found at index `idx`
/// * `None` if the byte is not in the range
///
/// # Safety
/// Caller must ensure `lo` and `hi` are valid indices into `byts`.
#[must_use]
pub fn tree_binary_search(byts: &[u8], lo: usize, hi: usize, c: u8) -> Option<usize> {
    use std::cmp::Ordering;

    if lo > hi {
        return None;
    }

    let mut lo = lo;
    let mut hi = hi;

    while lo < hi {
        let m = usize::midpoint(lo, hi);
        match byts[m].cmp(&c) {
            Ordering::Greater => {
                if m == 0 {
                    return None;
                }
                hi = m - 1;
            }
            Ordering::Less => {
                lo = m + 1;
            }
            Ordering::Equal => {
                return Some(m);
            }
        }
    }

    // Check if we found it at the final position
    if byts[lo] == c {
        Some(lo)
    } else {
        None
    }
}

/// Traverses a word tree to find word endings matching a given word.
///
/// This implements the core tree traversal algorithm from Neovim's find_word().
/// It walks the tree byte-by-byte, collecting all possible word endings.
///
/// # Arguments
/// * `byts` - The bytes array of the tree
/// * `idxs` - The indices array of the tree
/// * `word` - The word bytes to search for
/// * `max_len` - Maximum length to search (bytes available in word)
///
/// # Returns
/// A `TreeSearchResult` indicating whether the word was found and at what positions.
///
/// # Safety
/// This is a safe function that performs bounds checking on all array accesses.
#[must_use]
pub fn traverse_tree(byts: &[u8], idxs: &[IdxT], word: &[u8], max_len: usize) -> TreeSearchResult {
    if byts.is_empty() || idxs.is_empty() {
        return TreeSearchResult::Empty;
    }

    let mut arridx: usize = 0;
    let mut wlen: usize = 0;
    let mut endings: Vec<(usize, usize)> = Vec::new();

    loop {
        // Bounds check
        if arridx >= byts.len() {
            break;
        }

        // Get sibling count at this node
        let len = byts[arridx] as usize;
        arridx += 1;

        if arridx >= byts.len() {
            break;
        }

        // If the first possible byte is a zero, the word could end here.
        // Remember this ending, we check for the longest word first.
        if byts[arridx] == 0 {
            // Protect against corrupted spell files
            if endings.len() >= MAXWLEN {
                break;
            }

            endings.push((wlen, arridx));
            arridx += 1;
            let mut remaining = len.saturating_sub(1);

            // Skip over any additional zeros (same word, different flags/regions)
            while remaining > 0 && arridx < byts.len() && byts[arridx] == 0 {
                arridx += 1;
                remaining -= 1;
            }

            if remaining == 0 {
                // No more children, word must end here
                break;
            }
        }

        // Stop looking at end of the word
        if wlen >= word.len() || wlen >= max_len {
            break;
        }

        // Get the byte we're looking for
        let mut c = word[wlen];

        // Tab is treated like space
        if c == b'\t' {
            c = b' ';
        }

        // Calculate the range for binary search
        // We need to account for zeros we may have skipped
        let first_sibling = arridx;
        let remaining_siblings = len.saturating_sub(endings.last().map_or(0, |_| 1));

        if remaining_siblings == 0 {
            break;
        }

        let last_sibling = first_sibling + remaining_siblings - 1;

        if last_sibling >= byts.len() || last_sibling >= idxs.len() {
            break;
        }

        // Perform binary search in the sorted sibling list
        match tree_binary_search(byts, first_sibling, last_sibling, c) {
            Some(found_idx) => {
                // Continue at the child node
                if found_idx >= idxs.len() {
                    break;
                }
                arridx = idxs[found_idx] as usize;
                wlen += 1;

                // Handle multiple spaces: one space in the tree may match
                // multiple spaces in the word
                if c == b' ' {
                    while wlen < word.len() && wlen < max_len {
                        let next_c = word[wlen];
                        if next_c != b' ' && next_c != b'\t' {
                            break;
                        }
                        wlen += 1;
                    }
                }
            }
            None => {
                // No matching byte found
                break;
            }
        }
    }

    // Return appropriate result based on what we found
    match endings.len() {
        0 => TreeSearchResult::NotFound,
        1 => TreeSearchResult::Found {
            word_len: endings[0].0,
            end_idx: endings[0].1,
        },
        _ => TreeSearchResult::MultipleEndings { endings },
    }
}

/// Gets the flags for a word ending at the given index.
///
/// # Arguments
/// * `idxs` - The indices array of the tree
/// * `end_idx` - Index of the word end marker (from TreeSearchResult)
///
/// # Returns
/// The flags as a u32, or 0 if the index is out of bounds.
#[must_use]
pub fn get_word_flags(idxs: &[IdxT], end_idx: usize) -> u32 {
    if end_idx < idxs.len() {
        idxs[end_idx] as u32
    } else {
        0
    }
}

// =============================================================================
// Flag Checking Utilities
// =============================================================================

/// Checks if word flags indicate the word is banned.
#[must_use]
pub const fn is_banned(flags: u32) -> bool {
    (flags & WF_BANNED) != 0
}

/// Checks if word flags indicate the word is rare.
#[must_use]
pub const fn is_rare(flags: u32) -> bool {
    (flags & WF_RARE) != 0
}

/// Checks if word flags indicate the word requires compounding.
#[must_use]
pub const fn requires_compound(flags: u32) -> bool {
    (flags & WF_NEEDCOMP) != 0
}

/// Checks if word flags indicate the word can't start a compound.
#[must_use]
pub const fn no_compound_before(flags: u32) -> bool {
    (flags & WF_NOCOMPBEF) != 0
}

/// Checks if word flags indicate the word can't end a compound.
#[must_use]
pub const fn no_compound_after(flags: u32) -> bool {
    (flags & WF_NOCOMPAFT) != 0
}

/// Checks if word flags have a compound flag set.
#[must_use]
pub const fn has_compound_flag(flags: u32) -> bool {
    (flags & WF_COMPFLAG_MASK) != 0
}

/// Checks if a compound flag byte is in a string of allowed flags.
///
/// # Arguments
/// * `allowed_flags` - NUL-terminated string of allowed compound flag bytes
/// * `flag` - The compound flag byte to check
///
/// # Returns
/// true if the flag is in the allowed string
#[must_use]
pub fn compound_flag_allowed(allowed_flags: &[u8], flag: u8) -> bool {
    if flag == 0 {
        return false;
    }
    allowed_flags
        .iter()
        .take_while(|&&c| c != 0)
        .any(|&c| c == flag)
}

/// Gets all flags for a word that may have multiple flag/region combinations.
///
/// # Arguments
/// * `byts` - The bytes array of the tree
/// * `idxs` - The indices array of the tree
/// * `end_idx` - Index of the first word end marker
///
/// # Returns
/// A vector of all flags for this word.
#[must_use]
pub fn get_all_word_flags(byts: &[u8], idxs: &[IdxT], end_idx: usize) -> Vec<u32> {
    let mut flags = Vec::new();

    // Look backwards to find the sibling count
    if end_idx == 0 {
        return flags;
    }

    let mut idx = end_idx;

    // Collect all flags for consecutive zeros
    while idx < byts.len() && byts[idx] == 0 {
        if idx < idxs.len() {
            flags.push(idxs[idx] as u32);
        }
        idx += 1;
    }

    flags
}

// =============================================================================
// FFI Functions
// =============================================================================

/// FFI wrapper for tree_binary_search.
///
/// # Safety
///
/// `byts` must be a valid pointer to an array of at least `hi + 1` bytes.
/// `lo` must be <= `hi`.
#[no_mangle]
pub unsafe extern "C" fn rs_tree_binary_search(
    byts: *const u8,
    len: usize,
    lo: usize,
    hi: usize,
    c: u8,
) -> c_int {
    if byts.is_null() || hi >= len || lo > hi {
        return -1;
    }

    let byts_slice = std::slice::from_raw_parts(byts, len);

    tree_binary_search(byts_slice, lo, hi, c).map_or(-1, |idx| idx as c_int)
}

/// FFI wrapper that traverses a word tree and returns the longest match.
///
/// # Safety
///
/// - `byts` must be a valid pointer to an array of `tree_len` bytes
/// - `idxs` must be a valid pointer to an array of `tree_len` IdxT values
/// - `word` must be a valid pointer to an array of at least `word_len` bytes
/// - `out_end_idx` may be null; if non-null, receives the ending index
///
/// # Returns
/// - >= 0: Length of the longest matching word
/// - -1: Word not found
/// - -2: Empty tree
#[no_mangle]
pub unsafe extern "C" fn rs_traverse_tree(
    byts: *const u8,
    idxs: *const IdxT,
    tree_len: usize,
    word: *const u8,
    word_len: usize,
    out_end_idx: *mut usize,
) -> c_int {
    if byts.is_null() || idxs.is_null() || word.is_null() {
        return -2;
    }

    let byts_slice = std::slice::from_raw_parts(byts, tree_len);
    let idxs_slice = std::slice::from_raw_parts(idxs, tree_len);
    let word_slice = std::slice::from_raw_parts(word, word_len);

    match traverse_tree(byts_slice, idxs_slice, word_slice, word_len) {
        TreeSearchResult::Found { word_len, end_idx } => {
            if !out_end_idx.is_null() {
                *out_end_idx = end_idx;
            }
            word_len as c_int
        }
        TreeSearchResult::MultipleEndings { endings } => {
            // Return the longest match (last in the list)
            if let Some(&(len, idx)) = endings.last() {
                if !out_end_idx.is_null() {
                    *out_end_idx = idx;
                }
                len as c_int
            } else {
                -1
            }
        }
        TreeSearchResult::NotFound => -1,
        TreeSearchResult::Empty => -2,
    }
}

/// FFI wrapper to get word flags from a tree index.
///
/// # Safety
///
/// `idxs` must be a valid pointer to an array of at least `len` IdxT values.
#[no_mangle]
pub unsafe extern "C" fn rs_get_word_flags(idxs: *const IdxT, len: usize, end_idx: usize) -> u32 {
    if idxs.is_null() || end_idx >= len {
        return 0;
    }

    let idxs_slice = std::slice::from_raw_parts(idxs, len);
    get_word_flags(idxs_slice, end_idx)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_search_found() {
        let byts = [b'a', b'c', b'e', b'g', b'i'];

        assert_eq!(tree_binary_search(&byts, 0, 4, b'a'), Some(0));
        assert_eq!(tree_binary_search(&byts, 0, 4, b'c'), Some(1));
        assert_eq!(tree_binary_search(&byts, 0, 4, b'e'), Some(2));
        assert_eq!(tree_binary_search(&byts, 0, 4, b'g'), Some(3));
        assert_eq!(tree_binary_search(&byts, 0, 4, b'i'), Some(4));
    }

    #[test]
    fn test_binary_search_not_found() {
        let byts = [b'a', b'c', b'e', b'g', b'i'];

        assert_eq!(tree_binary_search(&byts, 0, 4, b'b'), None);
        assert_eq!(tree_binary_search(&byts, 0, 4, b'd'), None);
        assert_eq!(tree_binary_search(&byts, 0, 4, b'z'), None);
        assert_eq!(tree_binary_search(&byts, 0, 4, b'A'), None); // less than 'a'
    }

    #[test]
    fn test_binary_search_single_element() {
        let byts = [b'x'];

        assert_eq!(tree_binary_search(&byts, 0, 0, b'x'), Some(0));
        assert_eq!(tree_binary_search(&byts, 0, 0, b'y'), None);
    }

    #[test]
    fn test_binary_search_empty_range() {
        let byts = [b'a', b'b', b'c'];

        // lo > hi is an empty range
        assert_eq!(tree_binary_search(&byts, 2, 1, b'a'), None);
    }

    #[test]
    fn test_traverse_empty_tree() {
        let byts: &[u8] = &[];
        let idxs: &[IdxT] = &[];

        let result = traverse_tree(byts, idxs, b"test", 4);
        assert_eq!(result, TreeSearchResult::Empty);
    }

    #[test]
    fn test_traverse_single_word() {
        // Tree for word "a"
        // Node 0: 1 sibling ('a')
        // Node 3: 1 sibling (0 = word end)
        let byts: [u8; 4] = [
            1,    // sibling count
            b'a', // sibling byte
            1,    // child: 1 sibling
            0,    // word end
        ];
        let idxs: [IdxT; 4] = [
            0, // unused
            2, // 'a' -> child at index 2
            0, // unused
            0, // flags
        ];

        let result = traverse_tree(&byts, &idxs, b"a", 1);
        assert_eq!(
            result,
            TreeSearchResult::Found {
                word_len: 1,
                end_idx: 3
            }
        );
    }

    #[test]
    fn test_traverse_word_not_found() {
        // Tree for word "a"
        let byts: [u8; 4] = [1, b'a', 1, 0];
        let idxs: [IdxT; 4] = [0, 2, 0, 0];

        let result = traverse_tree(&byts, &idxs, b"b", 1);
        assert_eq!(result, TreeSearchResult::NotFound);
    }

    #[test]
    fn test_traverse_prefix_match() {
        // Tree for word "ab"
        let byts: [u8; 6] = [
            1,    // 1 sibling
            b'a', // 'a'
            1,    // child: 1 sibling
            b'b', // 'b'
            1,    // child: 1 sibling
            0,    // word end
        ];
        let idxs: [IdxT; 6] = [
            0, // unused
            2, // 'a' -> index 2
            0, // unused
            4, // 'b' -> index 4
            0, // unused
            0, // flags
        ];

        // Searching for "a" should not find "ab"
        let result = traverse_tree(&byts, &idxs, b"a", 1);
        assert_eq!(result, TreeSearchResult::NotFound);

        // Searching for "ab" should find it
        let result = traverse_tree(&byts, &idxs, b"ab", 2);
        assert_eq!(
            result,
            TreeSearchResult::Found {
                word_len: 2,
                end_idx: 5
            }
        );
    }

    #[test]
    fn test_traverse_space_collapsing() {
        // Tree for "a b" (single space in tree)
        let byts: [u8; 8] = [
            1,    // 1 sibling
            b'a', // 'a'
            1,    // child: 1 sibling
            b' ', // space
            1,    // child: 1 sibling
            b'b', // 'b'
            1,    // child: 1 sibling
            0,    // word end
        ];
        let idxs: [IdxT; 8] = [0, 2, 0, 4, 0, 6, 0, 0];

        // "a  b" (double space) should match "a b" (single space in tree)
        let result = traverse_tree(&byts, &idxs, b"a  b", 4);
        assert_eq!(
            result,
            TreeSearchResult::Found {
                word_len: 4, // All 4 bytes consumed
                end_idx: 7
            }
        );
    }

    #[test]
    fn test_traverse_tab_as_space() {
        // Tree for "a b"
        let byts: [u8; 8] = [1, b'a', 1, b' ', 1, b'b', 1, 0];
        let idxs: [IdxT; 8] = [0, 2, 0, 4, 0, 6, 0, 0];

        // "a\tb" should match "a b"
        let result = traverse_tree(&byts, &idxs, b"a\tb", 3);
        assert_eq!(
            result,
            TreeSearchResult::Found {
                word_len: 3,
                end_idx: 7
            }
        );
    }

    #[test]
    fn test_get_word_flags() {
        let idxs: [IdxT; 5] = [0, 100, 200, 300, 400];

        assert_eq!(get_word_flags(&idxs, 0), 0);
        assert_eq!(get_word_flags(&idxs, 1), 100);
        assert_eq!(get_word_flags(&idxs, 4), 400);
        assert_eq!(get_word_flags(&idxs, 10), 0); // out of bounds
    }

    #[test]
    fn test_get_all_word_flags() {
        // Tree where a word has multiple flag combinations
        // byts: [count, 0, 0, 0, ...] - three zeros = three flag combinations
        let byts: [u8; 5] = [3, 0, 0, 0, b'x'];
        let idxs: [IdxT; 5] = [0, 10, 20, 30, 99];

        let flags = get_all_word_flags(&byts, &idxs, 1);
        assert_eq!(flags, vec![10, 20, 30]);
    }

    #[test]
    fn test_ffi_binary_search() {
        let byts: [u8; 5] = [b'a', b'c', b'e', b'g', b'i'];

        unsafe {
            assert_eq!(rs_tree_binary_search(byts.as_ptr(), 5, 0, 4, b'e'), 2);
            assert_eq!(rs_tree_binary_search(byts.as_ptr(), 5, 0, 4, b'z'), -1);
            assert_eq!(rs_tree_binary_search(std::ptr::null(), 5, 0, 4, b'a'), -1);
        }
    }

    #[test]
    fn test_ffi_traverse_tree() {
        // Tree for word "hi"
        let byts: [u8; 6] = [1, b'h', 1, b'i', 1, 0];
        let idxs: [IdxT; 6] = [0, 2, 0, 4, 0, 42]; // flags = 42

        let mut end_idx: usize = 0;

        unsafe {
            let len = rs_traverse_tree(
                byts.as_ptr(),
                idxs.as_ptr(),
                6,
                b"hi".as_ptr(),
                2,
                &raw mut end_idx,
            );

            assert_eq!(len, 2);
            assert_eq!(end_idx, 5);

            let flags = rs_get_word_flags(idxs.as_ptr(), 6, end_idx);
            assert_eq!(flags, 42);
        }
    }

    #[test]
    fn test_ffi_traverse_tree_not_found() {
        let byts: [u8; 4] = [1, b'a', 1, 0];
        let idxs: [IdxT; 4] = [0, 2, 0, 0];

        unsafe {
            let len = rs_traverse_tree(
                byts.as_ptr(),
                idxs.as_ptr(),
                4,
                b"x".as_ptr(),
                1,
                std::ptr::null_mut(),
            );

            assert_eq!(len, -1);
        }
    }

    // =========================================================================
    // Flag utility tests
    // =========================================================================

    #[test]
    fn test_flag_constants() {
        assert_eq!(WF_REGION, 0x01);
        assert_eq!(WF_ONECAP, 0x02);
        assert_eq!(WF_ALLCAP, 0x04);
        assert_eq!(WF_RARE, 0x08);
        assert_eq!(WF_BANNED, 0x10);
        assert_eq!(WF_AFX, 0x20);
        assert_eq!(WF_FIXCAP, 0x40);
        assert_eq!(WF_KEEPCAP, 0x80);
    }

    #[test]
    fn test_get_compound_flag() {
        assert_eq!(get_compound_flag(0), 0);
        assert_eq!(get_compound_flag(0x0100_0000), 1);
        assert_eq!(get_compound_flag(0xFF00_0000), 255);
        assert_eq!(get_compound_flag(0xAB00_0000), 0xAB);
        // Lower bits should be ignored
        assert_eq!(get_compound_flag(0x4200_FFFF), 0x42);
    }

    #[test]
    fn test_is_banned() {
        assert!(!is_banned(0));
        assert!(!is_banned(WF_RARE));
        assert!(is_banned(WF_BANNED));
        assert!(is_banned(WF_BANNED | WF_RARE));
    }

    #[test]
    fn test_is_rare() {
        assert!(!is_rare(0));
        assert!(is_rare(WF_RARE));
        assert!(is_rare(WF_RARE | WF_REGION));
    }

    #[test]
    fn test_requires_compound() {
        assert!(!requires_compound(0));
        assert!(requires_compound(WF_NEEDCOMP));
        assert!(requires_compound(WF_NEEDCOMP | WF_RARE));
    }

    #[test]
    fn test_no_compound_before() {
        assert!(!no_compound_before(0));
        assert!(no_compound_before(WF_NOCOMPBEF));
    }

    #[test]
    fn test_no_compound_after() {
        assert!(!no_compound_after(0));
        assert!(no_compound_after(WF_NOCOMPAFT));
    }

    #[test]
    fn test_has_compound_flag() {
        assert!(!has_compound_flag(0));
        assert!(!has_compound_flag(0x00FF_FFFF));
        assert!(has_compound_flag(0x0100_0000));
        assert!(has_compound_flag(0xFF00_0000));
    }

    #[test]
    fn test_compound_flag_allowed() {
        let allowed = b"abc\0";

        assert!(compound_flag_allowed(allowed, b'a'));
        assert!(compound_flag_allowed(allowed, b'b'));
        assert!(compound_flag_allowed(allowed, b'c'));
        assert!(!compound_flag_allowed(allowed, b'd'));
        assert!(!compound_flag_allowed(allowed, b'A')); // case-sensitive
        assert!(!compound_flag_allowed(allowed, 0)); // NUL not allowed
    }

    #[test]
    fn test_compound_flag_allowed_empty() {
        let empty = b"\0";

        assert!(!compound_flag_allowed(empty, b'a'));
        assert!(!compound_flag_allowed(empty, 0));
    }
}
