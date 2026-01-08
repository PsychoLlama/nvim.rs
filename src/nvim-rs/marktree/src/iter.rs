//! Native Rust iterator operations for marktree.
//!
//! This module implements iterator traversal and lookup operations
//! using native Rust types from the `node` module.

// Allow various casts needed for B-tree level/index operations
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
// Allow pointer casts needed for NonNull
#![allow(clippy::ptr_cast_constness)]
#![allow(clippy::ref_as_ptr)]

use std::ptr::NonNull;

use crate::node::{MTNode, MarkTree, MarkTreeIter};
use crate::{compose, key_cmp, mt_lookup_id, relative, unrelative, MTKey, MTPos, MT_MAX_DEPTH};

// ============================================================================
// Iterator Validity and Position
// ============================================================================

impl MarkTreeIter {
    /// Position the iterator at the first key in the tree.
    ///
    /// Returns true if successful (tree non-empty), false otherwise.
    pub fn first(&mut self, tree: &MarkTree) -> bool {
        let Some(root) = tree.root.as_ref() else {
            self.invalidate();
            return false;
        };

        // Start at root and descend to leftmost leaf
        let mut current: NonNull<MTNode> =
            unsafe { NonNull::new_unchecked(root.as_ref() as *const MTNode as *mut MTNode) };
        let mut lvl = 0;
        self.pos = MTPos::new(0, 0);

        // Descend to leftmost leaf
        loop {
            let node = unsafe { current.as_ref() };
            if node.is_leaf() {
                break;
            }

            // Save state and descend
            self.s[lvl].i = 0;
            self.s[lvl].oldcol = self.pos.col;
            lvl += 1;

            if let Some(child) = node.children.as_ref().and_then(|c| c.ptr[0].as_ref()) {
                current = unsafe {
                    NonNull::new_unchecked(child.as_ref() as *const MTNode as *mut MTNode)
                };
            } else {
                // Should not happen in valid tree
                self.invalidate();
                return false;
            }
        }

        self.x = Some(current);
        self.i = 0;
        self.lvl = lvl as i32;
        true
    }

    /// Position the iterator at the last key in the tree.
    ///
    /// Returns true if successful (tree non-empty), false otherwise.
    pub fn last(&mut self, tree: &MarkTree) -> bool {
        let Some(root) = tree.root.as_ref() else {
            self.invalidate();
            return false;
        };

        // Start at root and descend to rightmost leaf
        let mut current: NonNull<MTNode> =
            unsafe { NonNull::new_unchecked(root.as_ref() as *const MTNode as *mut MTNode) };
        let mut lvl = 0;
        self.pos = MTPos::new(0, 0);

        // Descend to rightmost leaf
        loop {
            let node = unsafe { current.as_ref() };
            let n = node.n as usize;

            if node.is_leaf() {
                self.x = Some(current);
                self.i = (n as i32) - 1;
                self.lvl = lvl as i32;
                return true;
            }

            // Update position based on last key before descending
            if n > 0 {
                let oldcol = self.pos.col;
                self.s[lvl].oldcol = oldcol;
                let key = &node.keys[n - 1];
                compose(&mut self.pos, key.pos);
            }
            self.s[lvl].i = n as i32;
            lvl += 1;

            // Descend to last child
            if let Some(child) = node.children.as_ref().and_then(|c| c.ptr[n].as_ref()) {
                current = unsafe {
                    NonNull::new_unchecked(child.as_ref() as *const MTNode as *mut MTNode)
                };
            } else {
                self.invalidate();
                return false;
            }
        }
    }

    /// Move to the next key.
    ///
    /// Returns true if successful, false if at end.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> bool {
        let Some(mut x_ptr) = self.x else {
            return false;
        };

        self.i += 1;

        let node = unsafe { x_ptr.as_ref() };
        let level = node.level;
        let n = node.n;

        if level == 0 {
            // At leaf node
            if self.i < n {
                return true;
            }

            // Go up until we find a position with more keys
            loop {
                let current = unsafe { x_ptr.as_ref() };
                if self.i < current.n {
                    self.x = Some(x_ptr);
                    return true;
                }

                let Some(parent) = current.parent else {
                    self.invalidate();
                    return false;
                };

                self.lvl -= 1;
                let lvl = self.lvl as usize;
                self.i = self.s[lvl].i;

                // Adjust position
                if self.i > 0 {
                    let parent_ref = unsafe { parent.as_ref() };
                    let key = &parent_ref.keys[(self.i - 1) as usize];
                    self.pos.row -= key.pos.row;
                    self.pos.col = self.s[lvl].oldcol;
                }

                x_ptr = parent;
            }
        } else {
            // At internal node - descend to leftmost leaf of child
            self.descend_to_first_leaf()
        }
    }

    /// Move to the previous key.
    ///
    /// Returns true if successful, false if at beginning.
    pub fn prev(&mut self) -> bool {
        let Some(mut x_ptr) = self.x else {
            return false;
        };

        let node = unsafe { x_ptr.as_ref() };
        let level = node.level;

        if level == 0 {
            // At leaf node
            self.i -= 1;
            if self.i >= 0 {
                return true;
            }

            // Go up until we find a position before a key
            loop {
                let current = unsafe { x_ptr.as_ref() };

                let Some(parent) = current.parent else {
                    self.invalidate();
                    return false;
                };

                self.lvl -= 1;
                let lvl = self.lvl as usize;
                self.i = self.s[lvl].i - 1;

                // Adjust position
                if self.i >= 0 {
                    self.pos.col = self.s[lvl].oldcol;
                    self.x = Some(parent);
                    return true;
                }

                x_ptr = parent;
            }
        } else {
            // At internal node - descend to rightmost leaf of previous child
            self.descend_to_last_leaf()
        }
    }

    /// Helper to descend to the first leaf after moving to a child.
    fn descend_to_first_leaf(&mut self) -> bool {
        let Some(mut x_ptr) = self.x else {
            return false;
        };

        let mut i = self.i;

        loop {
            let node = unsafe { x_ptr.as_ref() };
            if node.is_leaf() {
                self.x = Some(x_ptr);
                self.i = 0;
                return node.n > 0;
            }

            // Save state and update position
            if i > 0 {
                let oldcol = self.pos.col;
                let lvl = self.lvl as usize;
                self.s[lvl].oldcol = oldcol;
                let key = &node.keys[(i - 1) as usize];
                compose(&mut self.pos, key.pos);
            }
            self.s[self.lvl as usize].i = i;
            self.lvl += 1;

            // Get child
            let child_idx = i as usize;
            if let Some(child) = node
                .children
                .as_ref()
                .and_then(|c| c.ptr[child_idx].as_ref())
            {
                x_ptr = unsafe {
                    NonNull::new_unchecked(child.as_ref() as *const MTNode as *mut MTNode)
                };
                i = 0;
            } else {
                self.invalidate();
                return false;
            }
        }
    }

    /// Helper to descend to the last leaf before moving to a child.
    fn descend_to_last_leaf(&mut self) -> bool {
        let Some(mut x_ptr) = self.x else {
            return false;
        };

        let mut i = self.i;

        loop {
            let node = unsafe { x_ptr.as_ref() };
            let n = node.n;

            if node.is_leaf() {
                self.x = Some(x_ptr);
                self.i = n - 1;
                return n > 0;
            }

            // Save state and update position
            let child_idx = i as usize;
            if child_idx > 0 {
                let oldcol = self.pos.col;
                let lvl = self.lvl as usize;
                self.s[lvl].oldcol = oldcol;
                let key = &node.keys[child_idx - 1];
                compose(&mut self.pos, key.pos);
            }
            self.s[self.lvl as usize].i = i;
            self.lvl += 1;

            // Get child and go to its rightmost position
            if let Some(child) = node
                .children
                .as_ref()
                .and_then(|c| c.ptr[child_idx].as_ref())
            {
                x_ptr = unsafe {
                    NonNull::new_unchecked(child.as_ref() as *const MTNode as *mut MTNode)
                };
                i = unsafe { x_ptr.as_ref() }.n;
            } else {
                self.invalidate();
                return false;
            }
        }
    }

    /// Get the current key with absolute position.
    #[must_use]
    pub fn current(&self) -> Option<MTKey> {
        let node = self.node()?;
        if self.i < 0 || self.i >= node.n {
            return None;
        }

        let mut key = node.keys[self.i as usize];
        unrelative(self.pos, &mut key.pos);
        Some(key)
    }

    /// Get the raw key at current position (relative position).
    #[must_use]
    pub fn raw_key(&self) -> Option<&MTKey> {
        let node = self.node()?;
        if self.i < 0 || self.i >= node.n {
            return None;
        }
        Some(&node.keys[self.i as usize])
    }

    /// Get the absolute position of the current key.
    #[must_use]
    pub fn position(&self) -> Option<MTPos> {
        let key = self.raw_key()?;
        let mut pos = key.pos;
        unrelative(self.pos, &mut pos);
        Some(pos)
    }

    /// Check if iterator is at the last key of current node.
    #[must_use]
    pub fn is_node_done(&self) -> bool {
        let Some(node) = self.node() else {
            return true;
        };
        self.i == node.n - 1
    }
}

// ============================================================================
// Search / Lookup
// ============================================================================

/// Binary search for a key position within a node.
///
/// Returns (position, found) where:
/// - position: index of key if found, or where it should be inserted
/// - found: true if exact match was found
#[must_use]
pub fn getp_aux(node: &MTNode, key: &MTKey) -> (i32, bool) {
    let n = node.n as usize;
    if n == 0 {
        return (-1, false);
    }

    let mut begin = 0;
    let mut end = n;

    while begin < end {
        let mid = (begin + end) >> 1;
        if key_cmp(&node.keys[mid], key) < 0 {
            begin = mid + 1;
        } else {
            end = mid;
        }
    }

    if begin == n {
        return ((n as i32) - 1, false);
    }

    let cmp = key_cmp(key, &node.keys[begin]);
    if cmp == 0 {
        (begin as i32, true)
    } else {
        ((begin as i32) - 1, false)
    }
}

impl MarkTreeIter {
    /// Position iterator at or before a specific position.
    ///
    /// If `right_gravity` is true, position after marks at the same position.
    /// Returns true if a key was found.
    pub fn get(&mut self, tree: &MarkTree, row: i32, col: i32, right_gravity: bool) -> bool {
        let Some(root) = tree.root.as_ref() else {
            self.invalidate();
            return false;
        };

        // Create search key
        let target = MTKey {
            pos: MTPos::new(row, col),
            ns: 0,
            id: 0,
            flags: if right_gravity {
                crate::flags::MT_FLAG_RIGHT_GRAVITY | crate::flags::MT_FLAG_LAST
            } else {
                0
            },
            decor_data: 0,
        };

        self.pos = MTPos::new(0, 0);
        self.lvl = 0;

        let mut current: NonNull<MTNode> =
            unsafe { NonNull::new_unchecked(root.as_ref() as *const MTNode as *mut MTNode) };
        let mut search_key = target;

        loop {
            let node = unsafe { current.as_ref() };
            let (i, found) = getp_aux(node, &search_key);

            if node.is_leaf() {
                // Found our leaf position
                // The position is where to start iterating from
                // If i is -1, position at 0; if found exact match stay at i; otherwise at i+1
                self.x = Some(current);
                if found {
                    self.i = i;
                } else if i < 0 {
                    self.i = 0;
                } else {
                    // i is valid but not exact match - key at i is < target, so go to i+1
                    // unless i+1 would be beyond the node
                    if i + 1 < node.n {
                        self.i = i + 1;
                    } else {
                        self.i = i;
                    }
                }
                return self.i < node.n;
            }

            // At internal node - need to descend
            let child_idx = if i >= 0 {
                // Position found or between keys
                if found {
                    // Exact match at internal node
                    self.x = Some(current);
                    self.i = i;
                    return true;
                }
                (i + 1) as usize
            } else {
                0
            };

            // Save state and update position
            if child_idx > 0 {
                let oldcol = self.pos.col;
                let lvl = self.lvl as usize;
                self.s[lvl].oldcol = oldcol;
                let key = &node.keys[child_idx - 1];
                compose(&mut self.pos, key.pos);
                // Make search key relative
                relative(key.pos, &mut search_key.pos);
            }
            self.s[self.lvl as usize].i = child_idx as i32;
            self.lvl += 1;

            // Descend to child
            if let Some(child) = node
                .children
                .as_ref()
                .and_then(|c| c.ptr[child_idx].as_ref())
            {
                current = unsafe {
                    NonNull::new_unchecked(child.as_ref() as *const MTNode as *mut MTNode)
                };
            } else {
                self.invalidate();
                return false;
            }
        }
    }
}

// ============================================================================
// Lookup by ID
// ============================================================================

impl MarkTree {
    /// Look up a mark by its namespace and ID.
    ///
    /// Returns the key if found, positioning the iterator at the mark.
    #[must_use]
    pub fn lookup_ns(
        &self,
        itr: Option<&mut MarkTreeIter>,
        ns: u32,
        id: u32,
        end: bool,
    ) -> Option<MTKey> {
        let lookup_id = mt_lookup_id(ns, id, end);
        self.lookup(itr, lookup_id)
    }

    /// Look up a mark by its combined lookup ID.
    ///
    /// Returns the key if found, positioning the iterator at the mark.
    #[must_use]
    pub fn lookup(&self, itr: Option<&mut MarkTreeIter>, id: u64) -> Option<MTKey> {
        let node_ptr = self.id2node.get(&id)?;
        let node = unsafe { node_ptr.as_ref() };

        // Find the key in the node
        let ns = (id >> 33) as u32;
        let mark_id = ((id >> 1) & 0xFFFF_FFFF) as u32;
        let end = (id & 1) != 0;

        for i in 0..node.n as usize {
            let key = &node.keys[i];
            if key.ns == ns && key.id == mark_id {
                let is_end = (key.flags & crate::flags::MT_FLAG_END) != 0;
                if is_end == end {
                    // Found it - set up iterator if provided
                    if let Some(iter) = itr {
                        iter.set_to_node(self, *node_ptr, i as i32);
                    }
                    return Some(*key);
                }
            }
        }

        None
    }
}

impl MarkTreeIter {
    /// Set iterator to point at a specific node and index.
    ///
    /// This recomputes the absolute position by walking up the tree.
    ///
    /// # Panics
    ///
    /// Panics if the node's parent chain is inconsistent (should not happen
    /// in a valid tree).
    #[allow(clippy::cast_lossless)]
    pub fn set_to_node(&mut self, _tree: &MarkTree, node: NonNull<MTNode>, idx: i32) {
        self.x = Some(node);
        self.i = idx;

        // Walk up to compute position and save state
        let mut current = node;
        let mut lvl = 0;
        let mut positions: [(MTPos, i32); MT_MAX_DEPTH] = [(MTPos::new(0, 0), 0); MT_MAX_DEPTH];

        // First pass: collect path to root
        loop {
            let n = unsafe { current.as_ref() };
            let p_idx = n.p_idx as i32;

            if let Some(parent) = n.parent {
                positions[lvl] = (MTPos::new(0, 0), p_idx);
                lvl += 1;
                current = parent;
            } else {
                break;
            }
        }

        // Second pass: compute positions from root down
        self.pos = MTPos::new(0, 0);
        self.lvl = lvl as i32;

        for l in (0..lvl).rev() {
            let parent_lvl = lvl - 1 - l;
            let (_, p_idx) = positions[l];

            self.s[parent_lvl].i = p_idx;
            self.s[parent_lvl].oldcol = self.pos.col;

            if p_idx > 0 {
                // Get parent node and add key position
                let mut walk = node;
                for _ in 0..l {
                    walk = unsafe { walk.as_ref() }.parent.unwrap();
                }
                let parent_node = unsafe { walk.as_ref() }.parent.unwrap();
                let parent_ref = unsafe { parent_node.as_ref() };
                let key = &parent_ref.keys[(p_idx - 1) as usize];
                compose(&mut self.pos, key.pos);
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

    fn create_test_tree() -> MarkTree {
        let mut tree = MarkTree::new();

        // Create a simple leaf node with some keys
        let mut root = MTNode::new_leaf();
        root.keys[0] = MTKey {
            pos: MTPos::new(0, 5),
            ns: 1,
            id: 1,
            flags: crate::flags::MT_FLAG_REAL,
            decor_data: 0,
        };
        root.keys[1] = MTKey {
            pos: MTPos::new(0, 10),
            ns: 1,
            id: 2,
            flags: crate::flags::MT_FLAG_REAL,
            decor_data: 0,
        };
        root.keys[2] = MTKey {
            pos: MTPos::new(1, 0),
            ns: 1,
            id: 3,
            flags: crate::flags::MT_FLAG_REAL,
            decor_data: 0,
        };
        root.n = 3;

        tree.root = Some(Box::new(root));
        tree.n_keys = 3;
        tree.n_nodes = 1;

        tree
    }

    #[test]
    fn test_iter_first() {
        let tree = create_test_tree();
        let mut iter = MarkTreeIter::new();

        assert!(iter.first(&tree));
        assert!(iter.is_valid());

        let key = iter.current().unwrap();
        assert_eq!(key.pos.row, 0);
        assert_eq!(key.pos.col, 5);
        assert_eq!(key.id, 1);
    }

    #[test]
    fn test_iter_last() {
        let tree = create_test_tree();
        let mut iter = MarkTreeIter::new();

        assert!(iter.last(&tree));
        assert!(iter.is_valid());

        let key = iter.current().unwrap();
        assert_eq!(key.pos.row, 1);
        assert_eq!(key.pos.col, 0);
        assert_eq!(key.id, 3);
    }

    #[test]
    fn test_iter_next() {
        let tree = create_test_tree();
        let mut iter = MarkTreeIter::new();

        assert!(iter.first(&tree));

        // First key
        let key1 = iter.current().unwrap();
        assert_eq!(key1.id, 1);

        // Second key
        assert!(iter.next());
        let key2 = iter.current().unwrap();
        assert_eq!(key2.id, 2);

        // Third key
        assert!(iter.next());
        let key3 = iter.current().unwrap();
        assert_eq!(key3.id, 3);

        // End
        assert!(!iter.next());
        assert!(!iter.is_valid());
    }

    #[test]
    fn test_iter_prev() {
        let tree = create_test_tree();
        let mut iter = MarkTreeIter::new();

        assert!(iter.last(&tree));

        // Last key
        let key3 = iter.current().unwrap();
        assert_eq!(key3.id, 3);

        // Second key
        assert!(iter.prev());
        let key2 = iter.current().unwrap();
        assert_eq!(key2.id, 2);

        // First key
        assert!(iter.prev());
        let key1 = iter.current().unwrap();
        assert_eq!(key1.id, 1);

        // Beginning
        assert!(!iter.prev());
        assert!(!iter.is_valid());
    }

    #[test]
    fn test_getp_aux() {
        let mut node = MTNode::new_leaf();
        node.keys[0] = MTKey {
            pos: MTPos::new(0, 5),
            ns: 0,
            id: 0,
            flags: 0,
            decor_data: 0,
        };
        node.keys[1] = MTKey {
            pos: MTPos::new(0, 10),
            ns: 0,
            id: 0,
            flags: 0,
            decor_data: 0,
        };
        node.n = 2;

        // Search before first
        let search1 = MTKey {
            pos: MTPos::new(0, 0),
            ..MTKey::default()
        };
        let (pos1, found1) = getp_aux(&node, &search1);
        assert_eq!(pos1, -1);
        assert!(!found1);

        // Search at first
        let search2 = MTKey {
            pos: MTPos::new(0, 5),
            ..MTKey::default()
        };
        let (pos2, found2) = getp_aux(&node, &search2);
        assert_eq!(pos2, 0);
        assert!(found2);

        // Search between
        let search3 = MTKey {
            pos: MTPos::new(0, 7),
            ..MTKey::default()
        };
        let (pos3, found3) = getp_aux(&node, &search3);
        assert_eq!(pos3, 0);
        assert!(!found3);

        // Search after last
        let search4 = MTKey {
            pos: MTPos::new(0, 15),
            ..MTKey::default()
        };
        let (pos4, found4) = getp_aux(&node, &search4);
        assert_eq!(pos4, 1);
        assert!(!found4);
    }

    #[test]
    fn test_iter_get() {
        let tree = create_test_tree();
        let mut iter = MarkTreeIter::new();

        // Get at exact position
        assert!(iter.get(&tree, 0, 5, false));
        let key = iter.current().unwrap();
        assert_eq!(key.id, 1);

        // Get at position between keys - should find first key >= position
        assert!(iter.get(&tree, 0, 7, false));
        let key = iter.current().unwrap();
        assert_eq!(key.id, 2); // First key at or after (0,7) is (0,10) with id=2

        // Get at second row
        assert!(iter.get(&tree, 1, 0, false));
        let key = iter.current().unwrap();
        assert_eq!(key.id, 3);
    }

    #[test]
    fn test_empty_tree() {
        let tree = MarkTree::new();
        let mut iter = MarkTreeIter::new();

        assert!(!iter.first(&tree));
        assert!(!iter.is_valid());

        assert!(!iter.last(&tree));
        assert!(!iter.is_valid());
    }
}
