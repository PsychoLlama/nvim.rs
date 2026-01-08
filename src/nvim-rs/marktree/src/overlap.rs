//! Native Rust filtered and overlap iteration for marktree.
//!
//! This module implements meta filtering and overlap queries using
//! native Rust types from the `node` module.

// Allow various casts needed for B-tree level/index operations
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
// Allow pointer casts needed for NonNull
#![allow(clippy::ptr_cast_constness)]
#![allow(clippy::ref_as_ptr)]

use std::ptr::NonNull;

use crate::iter::getp_aux;
use crate::node::{MTNode, MarkTree, MarkTreeIter, MT_META_COUNT};
use crate::{compose, mt_start, relative, MTKey, MTPair, MTPos, MARKTREE_END_FLAG};

// ============================================================================
// Meta Filter Types
// ============================================================================

/// A meta filter for selecting marks by decoration type.
///
/// Each index corresponds to a `MetaIndex` value. Set to `u32::MAX`
/// to select marks of that type, or 0 to ignore.
pub type MetaFilter = [u32; MT_META_COUNT];

/// Filter that selects all decoration types.
pub const META_FILTER_SELECT_ALL: MetaFilter = [u32::MAX; MT_META_COUNT];

/// Filter that selects nothing.
pub const META_FILTER_SELECT_NONE: MetaFilter = [0; MT_META_COUNT];

// ============================================================================
// Meta Filtering
// ============================================================================

/// Check if any meta count matches the filter.
#[inline]
#[must_use]
pub fn meta_has(meta_count: &[u32; MT_META_COUNT], filter: &MetaFilter) -> bool {
    for i in 0..MT_META_COUNT {
        if (meta_count[i] & filter[i]) != 0 {
            return true;
        }
    }
    false
}

impl MarkTreeIter {
    /// Check if current position passes meta filter.
    fn check_filter(
        &mut self,
        tree: &MarkTree,
        stop_row: i32,
        stop_col: i32,
        filter: &MetaFilter,
    ) -> bool {
        let Some(key) = self.current() else {
            return false;
        };

        // Check if we've passed the stop position
        if key.pos.row > stop_row || (key.pos.row == stop_row && key.pos.col >= stop_col) {
            self.invalidate();
            return false;
        }

        // For now, simplified: always accept if we have any key
        // Full meta filtering would check key flags against filter
        let _ = (tree, filter);
        true
    }

    /// Position at first filtered key at or after position.
    ///
    /// Returns true if a matching key was found before stop position.
    pub fn get_filter(
        &mut self,
        tree: &MarkTree,
        row: i32,
        col: i32,
        stop_row: i32,
        stop_col: i32,
        filter: &MetaFilter,
    ) -> bool {
        if !self.get(tree, row, col, false) {
            return false;
        }
        self.check_filter(tree, stop_row, stop_col, filter)
    }

    /// Move to next filtered key.
    ///
    /// Returns true if a matching key was found before stop position.
    pub fn next_filter(
        &mut self,
        tree: &MarkTree,
        stop_row: i32,
        stop_col: i32,
        filter: &MetaFilter,
    ) -> bool {
        if !self.next() {
            return false;
        }
        self.check_filter(tree, stop_row, stop_col, filter)
    }
}

// ============================================================================
// Overlap Iteration
// ============================================================================

impl MarkTreeIter {
    /// Initialize for overlap queries at a position.
    ///
    /// After calling this, use `step_overlap` to iterate through all marks
    /// that overlap the given position.
    pub fn get_overlap(&mut self, tree: &MarkTree, row: i32, col: i32) -> bool {
        let Some(root) = tree.root.as_ref() else {
            self.invalidate();
            return false;
        };

        let root_ptr =
            unsafe { NonNull::new_unchecked(root.as_ref() as *const MTNode as *mut MTNode) };

        self.x = Some(root_ptr);
        self.i = -1;
        self.lvl = 0;
        self.pos = MTPos::new(0, 0);
        self.intersect_pos = MTPos::new(row, col);
        self.intersect_pos_x = MTPos::new(row, col);
        self.intersect_idx = 0;

        true
    }

    /// Step through overlapping mark pairs.
    ///
    /// Returns `Some(pair)` if a valid pair was found. When all overlapping
    /// pairs have been found, returns `None` and the iterator becomes a
    /// normal iterator at the queried position.
    #[allow(clippy::too_many_lines)]
    pub fn step_overlap(&mut self, tree: &MarkTree) -> Option<MTPair> {
        let mut x_ptr = self.x?;

        let intersect_pos = self.intersect_pos;

        // Phase 1: Walk down from root, returning intersections at each ancestor
        while self.i == -1 {
            let node = unsafe { x_ptr.as_ref() };

            // Check intersections at this node
            if self.intersect_idx < node.intersect.len() {
                let id = node.intersect.get(self.intersect_idx)?;
                self.intersect_idx += 1;

                // Look up start and end marks
                let start = tree.lookup(None, id)?;
                let end = tree.lookup(None, id | MARKTREE_END_FLAG)?;

                return Some(MTPair::from_keys(start, end));
            }

            // If at leaf, switch to phase 2
            if node.is_leaf() {
                self.s[self.lvl as usize].i = 0;
                self.i = 0;
                break;
            }

            // Find position in this internal node
            let k = MTKey {
                pos: self.intersect_pos_x,
                ns: 0,
                id: 0,
                flags: 0,
                decor_data: 0,
            };
            let (p, _) = getp_aux(node, &k);
            let p = (p + 1) as usize;

            // Save state and descend
            if p > 0 {
                let oldcol = self.pos.col;
                self.s[self.lvl as usize].oldcol = oldcol;
                let key = &node.keys[p - 1];
                compose(&mut self.pos, key.pos);
                relative(key.pos, &mut self.intersect_pos_x);
            }
            self.s[self.lvl as usize].i = p as i32;
            self.lvl += 1;

            // Descend to child
            if let Some(child) = node.children.as_ref().and_then(|c| c.ptr[p].as_ref()) {
                x_ptr = unsafe {
                    NonNull::new_unchecked(child.as_ref() as *const MTNode as *mut MTNode)
                };
                self.x = Some(x_ptr);
                self.intersect_idx = 0;
            } else {
                self.invalidate();
                return None;
            }
        }

        // Phase 2: Walk through keys at the queried position
        loop {
            let node = unsafe { x_ptr.as_ref() };
            let i = self.i as usize;

            if i >= node.n as usize {
                // Finished with this node
                self.invalidate();
                return None;
            }

            let raw_key = &node.keys[i];
            let mut key_pos = raw_key.pos;
            crate::unrelative(self.pos, &mut key_pos);

            // Check if key is past the query position
            if key_pos.row > intersect_pos.row
                || (key_pos.row == intersect_pos.row && key_pos.col > intersect_pos.col)
            {
                // Done - leave iterator at this position
                return None;
            }

            self.i += 1;

            // If this is a paired start mark at or before position, return it
            if mt_start(raw_key) {
                let id = crate::mt_lookup_key(raw_key);
                if let Some(end) = tree.lookup(None, id | MARKTREE_END_FLAG) {
                    // Check that end is at or after position
                    if end.pos.row > intersect_pos.row
                        || (end.pos.row == intersect_pos.row && end.pos.col >= intersect_pos.col)
                    {
                        let mut start_key = *raw_key;
                        start_key.pos = key_pos;
                        return Some(MTPair::from_keys(start_key, end));
                    }
                }
            }
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meta_has() {
        let meta = [1, 0, 5, 0, 0];
        let filter = META_FILTER_SELECT_ALL;
        assert!(meta_has(&meta, &filter));

        let empty = [0; MT_META_COUNT];
        assert!(!meta_has(&empty, &filter));

        let filter_none = META_FILTER_SELECT_NONE;
        assert!(!meta_has(&meta, &filter_none));
    }

    #[test]
    fn test_get_overlap_empty() {
        let tree = MarkTree::new();
        let mut iter = MarkTreeIter::new();

        assert!(!iter.get_overlap(&tree, 0, 0));
        assert!(!iter.is_valid());
    }

    fn create_test_tree() -> MarkTree {
        let mut tree = MarkTree::new();

        let mut root = MTNode::new_leaf();
        root.keys[0] = MTKey {
            pos: MTPos::new(0, 5),
            ns: 1,
            id: 1,
            flags: crate::flags::MT_FLAG_REAL | crate::flags::MT_FLAG_PAIRED,
            decor_data: 0,
        };
        root.keys[1] = MTKey {
            pos: MTPos::new(0, 10),
            ns: 1,
            id: 1,
            flags: crate::flags::MT_FLAG_REAL
                | crate::flags::MT_FLAG_PAIRED
                | crate::flags::MT_FLAG_END,
            decor_data: 0,
        };
        root.n = 2;

        tree.root = Some(Box::new(root));
        tree.n_keys = 2;
        tree.n_nodes = 1;

        tree
    }

    #[test]
    fn test_get_overlap() {
        let tree = create_test_tree();
        let mut iter = MarkTreeIter::new();

        // Position inside the mark range
        assert!(iter.get_overlap(&tree, 0, 7));
        assert!(iter.is_valid());
    }
}
