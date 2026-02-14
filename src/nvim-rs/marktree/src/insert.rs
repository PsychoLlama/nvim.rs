//! Native Rust B-tree insertion operations for marktree.
//!
//! This module implements insertion with node splitting using
//! native Rust types from the `node` module.

// Allow various casts needed for B-tree level/index operations
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
// Allow pointer casts needed for NonNull
#![allow(clippy::ptr_cast_constness)]
#![allow(clippy::ref_as_ptr)]
// Allow matching C naming conventions
#![allow(clippy::many_single_char_names)]
// Allow range loops for meta arrays - clearer for parallel arrays
#![allow(clippy::needless_range_loop)]
// Allow missing panics doc in internal functions
#![allow(clippy::missing_panics_doc)]

use std::ptr::NonNull;

use crate::iter::getp_aux;
use crate::node::{MTNode, MTNodeChildren, MarkTree, MT_META_COUNT};
use crate::{key_cmp, relative, unrelative, MTKey, MTPos, MT_BRANCH_FACTOR};

/// Branch factor for the B-tree.
const T: usize = MT_BRANCH_FACTOR;

// ============================================================================
// Meta Description
// ============================================================================

/// Describe meta counts for a key.
///
/// Fills `meta_inc` with the meta contributions of the key based on its flags.
pub fn meta_describe_key(key: &MTKey, meta_inc: &mut [u32; MT_META_COUNT]) {
    meta_inc.fill(0);

    // Map flags to meta indices
    if key.flags & crate::flags::MT_FLAG_DECOR_VIRT_TEXT_INLINE != 0 {
        meta_inc[0] = 1; // kMTMetaInline
    }
    if key.flags & crate::flags::MT_FLAG_DECOR_VIRT_LINES != 0 {
        meta_inc[1] = 1; // kMTMetaLines
    }
    if key.flags & crate::flags::MT_FLAG_DECOR_SIGNHL != 0 {
        meta_inc[2] = 1; // kMTMetaSignHL
    }
    if key.flags & crate::flags::MT_FLAG_DECOR_SIGNTEXT != 0 {
        meta_inc[3] = 1; // kMTMetaSignText
    }
    if key.flags & crate::flags::MT_FLAG_DECOR_CONCEAL_LINES != 0 {
        meta_inc[4] = 1; // kMTMetaConcealLines
    }
}

/// Describe meta counts for an entire node.
pub fn meta_describe_node(node: &MTNode, meta_out: &mut [u32; MT_META_COUNT]) {
    meta_out.fill(0);

    // Sum meta from all keys
    let mut key_meta = [0u32; MT_META_COUNT];
    for idx in 0..node.n as usize {
        meta_describe_key(&node.keys[idx], &mut key_meta);
        for m in 0..MT_META_COUNT {
            meta_out[m] += key_meta[m];
        }
    }

    // If internal node, add meta from children
    if !node.is_leaf() {
        if let Some(ref children) = node.children {
            for idx in 0..=node.n as usize {
                for m in 0..MT_META_COUNT {
                    meta_out[m] += children.meta[idx][m];
                }
            }
        }
    }
}

// ============================================================================
// Insertion into Leaf
// ============================================================================

/// Insert a key into a leaf node at position i.
fn insert_into_leaf(node: &mut MTNode, pos: usize, key: MTKey) {
    debug_assert!(node.is_leaf());
    debug_assert!((node.n as usize) < 2 * T - 1);

    let n = node.n as usize;
    // Shift keys right
    for j in (pos..n).rev() {
        node.keys[j + 1] = node.keys[j];
    }
    node.keys[pos] = key;
    node.n += 1;
}

// ============================================================================
// Node Splitting
// ============================================================================

/// Split a full leaf node.
///
/// Returns the new right node and the key to promote to parent.
fn split_leaf(node: &mut MTNode) -> (MTNode, MTKey) {
    debug_assert!(node.is_leaf());
    debug_assert_eq!(node.n as usize, 2 * T - 1);

    let mut right = MTNode::new_leaf();

    // Copy right half of keys to new node
    for j in 0..(T - 1) {
        right.keys[j] = node.keys[T + j];
    }
    right.n = (T - 1) as i32;

    // Copy intersection list
    right.intersect = node.intersect.clone();

    // The key to promote
    let promote_key = node.keys[T - 1];

    // Shrink original node
    node.n = (T - 1) as i32;

    (right, promote_key)
}

/// Split a full internal node.
///
/// Returns the new right node and the key to promote to parent.
fn split_internal(node: &mut MTNode) -> (MTNode, MTKey) {
    debug_assert!(!node.is_leaf());
    debug_assert_eq!(node.n as usize, 2 * T - 1);

    let mut right = MTNode::new_internal(node.level);

    // Copy right half of keys to new node
    for j in 0..(T - 1) {
        right.keys[j] = node.keys[T + j];
    }
    right.n = (T - 1) as i32;

    // Copy intersection list
    right.intersect = node.intersect.clone();

    // Copy children and their meta
    let node_children = node.children.as_mut().unwrap();
    let right_children = right
        .children
        .get_or_insert_with(|| Box::new(MTNodeChildren::new()));

    for j in 0..T {
        right_children.ptr[j] = node_children.ptr[T + j].take();
        right_children.meta[j] = node_children.meta[T + j];
    }

    // The key to promote
    let promote_key = node.keys[T - 1];

    // Shrink original node
    node.n = (T - 1) as i32;

    (right, promote_key)
}

// ============================================================================
// Tree Insertion
// ============================================================================

impl MarkTree {
    /// Insert a key into the tree.
    pub fn put_key(&mut self, key: MTKey) {
        if self.root.is_none() {
            // Create new root
            let mut root = MTNode::new_leaf();
            root.keys[0] = key;
            root.n = 1;
            self.root = Some(Box::new(root));
            self.n_keys = 1;
            self.n_nodes = 1;

            // Register in id2node
            let root_ptr = unsafe {
                NonNull::new_unchecked(
                    self.root.as_ref().unwrap().as_ref() as *const MTNode as *mut MTNode
                )
            };
            self.register_key(&key, root_ptr);

            // Update meta root
            let mut meta_inc = [0u32; MT_META_COUNT];
            meta_describe_key(&key, &mut meta_inc);
            for m in 0..MT_META_COUNT {
                self.meta_root[m] += meta_inc[m];
            }
            return;
        }

        // Check if root is full
        let root_is_full = self
            .root
            .as_ref()
            .is_some_and(|r| r.n as usize >= 2 * T - 1);
        if root_is_full {
            // Need to split root - create new root first
            let mut old_root = self.root.take().unwrap();
            let old_level = old_root.level;
            let is_leaf = old_root.is_leaf();

            // Split the old root
            let (right_node, promote_key) = if is_leaf {
                split_leaf(&mut old_root)
            } else {
                split_internal(&mut old_root)
            };
            let left_node = old_root;

            // Create new internal root
            let mut new_root = MTNode::new_internal(old_level + 1);
            new_root.keys[0] = promote_key;
            new_root.n = 1;

            // Set up children
            let children = new_root.children.as_mut().unwrap();
            children.ptr[0] = Some(left_node);
            children.ptr[1] = Some(Box::new(right_node));

            self.root = Some(Box::new(new_root));
            self.n_nodes += 2; // new root + right split node

            // Update parent pointers
            let root_ptr = unsafe {
                NonNull::new_unchecked(
                    self.root.as_ref().unwrap().as_ref() as *const MTNode as *mut MTNode
                )
            };
            let children = self.root.as_mut().unwrap().children.as_mut().unwrap();
            if let Some(ref mut left) = children.ptr[0] {
                left.parent = Some(root_ptr);
                left.p_idx = 0;
            }
            if let Some(ref mut right) = children.ptr[1] {
                right.parent = Some(root_ptr);
                right.p_idx = 1;
            }

            // Calculate meta for children
            let mut meta = [0u32; MT_META_COUNT];
            if let Some(ref left) = children.ptr[0] {
                meta_describe_node(left, &mut meta);
                children.meta[0] = meta;
            }
            if let Some(ref right) = children.ptr[1] {
                meta_describe_node(right, &mut meta);
                children.meta[1] = meta;
            }
        }

        // Now insert into non-full tree
        let mut meta_inc = [0u32; MT_META_COUNT];
        meta_describe_key(&key, &mut meta_inc);

        self.insert_non_full(key, &meta_inc);

        self.n_keys += 1;

        // Update meta root
        for m in 0..MT_META_COUNT {
            self.meta_root[m] += meta_inc[m];
        }
    }

    /// Insert key into a non-full tree (root is not full).
    ///
    /// Uses an iterative approach with raw pointers to avoid borrow checker issues.
    fn insert_non_full(&mut self, key: MTKey, meta_inc: &[u32; MT_META_COUNT]) {
        // We use raw pointers to navigate the tree iteratively
        // This is safe because:
        // 1. We maintain tree invariants
        // 2. We don't create aliasing mutable references
        // 3. All pointer accesses are within valid tree nodes

        let mut current_key = key;
        let mut path: Vec<(NonNull<MTNode>, usize)> = Vec::new();

        // Get raw pointer to root
        let root_ptr =
            unsafe { NonNull::new_unchecked(self.root.as_mut().unwrap().as_mut() as *mut MTNode) };

        let mut node_ptr = root_ptr;

        loop {
            let node = unsafe { node_ptr.as_mut() };

            let (pos, _) = getp_aux(node, &current_key);
            let insert_pos = (pos + 1) as usize;

            if node.is_leaf() {
                // Insert directly into leaf
                insert_into_leaf(node, insert_pos, current_key);

                // Register in id2node
                self.register_key(&current_key, node_ptr);

                // Update meta along the path
                for (parent_ptr, child_idx) in path.iter().rev() {
                    let parent = unsafe { parent_ptr.as_ref() };
                    if parent.children.is_some() {
                        // We need to update meta, but children is immutable here
                        // Use raw pointer to update
                        let parent_mut = unsafe { (*parent_ptr).as_ptr().as_mut().unwrap() };
                        let children_mut = parent_mut.children.as_mut().unwrap();
                        for m in 0..MT_META_COUNT {
                            children_mut.meta[*child_idx][m] += meta_inc[m];
                        }
                    }
                }

                break;
            }

            // Internal node - check if child is full
            let children = node.children.as_mut().unwrap();
            let child = children.ptr[insert_pos].as_mut().expect("child must exist");

            if child.n as usize >= 2 * T - 1 {
                // Split child
                let is_leaf = child.is_leaf();
                let (mut right_node, promote_key) = if is_leaf {
                    split_leaf(child)
                } else {
                    split_internal(child)
                };

                // Insert promote_key and right_node into current node
                let n = node.n as usize;

                // Make room for new key
                for j in (insert_pos..n).rev() {
                    node.keys[j + 1] = node.keys[j];
                }

                // Make promote_key relative if needed
                let mut rel_promote = promote_key;
                if insert_pos > 0 {
                    unrelative(node.keys[insert_pos - 1].pos, &mut rel_promote.pos);
                }
                node.keys[insert_pos] = rel_promote;

                // Make room for new child
                for j in (insert_pos + 1..=n).rev() {
                    children.ptr[j + 1] = children.ptr[j].take();
                    children.meta[j + 1] = children.meta[j];
                }

                // Update parent pointer for right node
                right_node.parent = Some(node_ptr);
                right_node.p_idx = (insert_pos + 1) as i16;

                children.ptr[insert_pos + 1] = Some(Box::new(right_node));

                // Calculate meta for new child
                let mut meta = [0u32; MT_META_COUNT];
                if let Some(ref right) = children.ptr[insert_pos + 1] {
                    meta_describe_node(right, &mut meta);
                    children.meta[insert_pos + 1] = meta;
                }

                // Update p_idx for shifted children
                for j in insert_pos + 2..=n + 1 {
                    if let Some(ref mut c) = children.ptr[j] {
                        c.p_idx = j as i16;
                    }
                }

                node.n += 1;
                self.n_nodes += 1;

                // Make keys in right child relative to promote_key
                if let Some(ref mut right) = children.ptr[insert_pos + 1] {
                    for k in 0..right.n as usize {
                        relative(promote_key.pos, &mut right.keys[k].pos);
                    }
                }

                // Recalculate meta for left child
                if let Some(ref left) = children.ptr[insert_pos] {
                    meta_describe_node(left, &mut meta);
                    children.meta[insert_pos] = meta;
                }

                // Decide which child to descend into
                let cmp = key_cmp(&current_key, &promote_key);
                let target = if cmp > 0 { insert_pos + 1 } else { insert_pos };

                // Make key relative
                if target > 0 {
                    relative(node.keys[target - 1].pos, &mut current_key.pos);
                }

                // Record path and descend
                path.push((node_ptr, target));
                let target_child = children.ptr[target].as_mut().unwrap();
                node_ptr = unsafe { NonNull::new_unchecked(target_child.as_mut() as *mut MTNode) };
            } else {
                // Child not full - just descend
                if insert_pos > 0 {
                    relative(node.keys[insert_pos - 1].pos, &mut current_key.pos);
                }

                // Record path and descend
                path.push((node_ptr, insert_pos));
                let target_child = children.ptr[insert_pos].as_mut().unwrap();
                node_ptr = unsafe { NonNull::new_unchecked(target_child.as_mut() as *mut MTNode) };
            }
        }
    }

    /// Insert a mark with optional paired end.
    pub fn put(&mut self, key: MTKey, end_row: i32, end_col: i32, end_right: bool) {
        let mut key = key;

        if end_row >= 0 {
            key.flags |= crate::flags::MT_FLAG_PAIRED;
        }

        self.put_key(key);

        if end_row >= 0 {
            // Insert end key
            let mut end_key = key;
            end_key.flags = (key.flags & !crate::flags::MT_FLAG_RIGHT_GRAVITY)
                | crate::flags::MT_FLAG_END
                | if end_right {
                    crate::flags::MT_FLAG_RIGHT_GRAVITY
                } else {
                    0
                };
            end_key.pos = MTPos::new(end_row, end_col);
            self.put_key(end_key);
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DecorInlineData;

    #[test]
    fn test_put_single() {
        let mut tree = MarkTree::new();

        let key = MTKey {
            pos: MTPos::new(0, 5),
            ns: 1,
            id: 1,
            flags: crate::flags::MT_FLAG_REAL,
            decor_data: DecorInlineData::zero(),
        };

        tree.put_key(key);

        assert_eq!(tree.n_keys, 1);
        assert!(tree.root.is_some());
        let root = tree.root.as_ref().unwrap();
        assert_eq!(root.n, 1);
        assert_eq!(root.keys[0].id, 1);
    }

    #[test]
    fn test_put_multiple() {
        let mut tree = MarkTree::new();

        for id in 0..10 {
            let key = MTKey {
                pos: MTPos::new(0, id * 5),
                ns: 1,
                id: id as u32 + 1,
                flags: crate::flags::MT_FLAG_REAL,
                decor_data: DecorInlineData::zero(),
            };
            tree.put_key(key);
        }

        assert_eq!(tree.n_keys, 10);
    }

    #[test]
    fn test_put_requires_split() {
        let mut tree = MarkTree::new();

        // Insert enough keys to require splitting (more than 2*T-1 = 19 keys)
        for id in 0..25 {
            let key = MTKey {
                pos: MTPos::new(0, id * 5),
                ns: 1,
                id: id as u32 + 1,
                flags: crate::flags::MT_FLAG_REAL,
                decor_data: DecorInlineData::zero(),
            };
            tree.put_key(key);
        }

        assert_eq!(tree.n_keys, 25);

        // Tree should have split - root should be internal
        let root = tree.root.as_ref().unwrap();
        assert!(!root.is_leaf());
    }

    #[test]
    fn test_meta_describe_key() {
        let key = MTKey {
            pos: MTPos::new(0, 0),
            ns: 1,
            id: 1,
            flags: crate::flags::MT_FLAG_DECOR_VIRT_TEXT_INLINE
                | crate::flags::MT_FLAG_DECOR_SIGNTEXT,
            decor_data: DecorInlineData::zero(),
        };

        let mut meta = [0u32; MT_META_COUNT];
        meta_describe_key(&key, &mut meta);

        assert_eq!(meta[0], 1); // Inline
        assert_eq!(meta[3], 1); // SignText
    }

    #[test]
    fn test_insert_into_leaf() {
        let mut node = MTNode::new_leaf();

        let key1 = MTKey {
            pos: MTPos::new(0, 10),
            ns: 1,
            id: 1,
            flags: 0,
            decor_data: DecorInlineData::zero(),
        };
        insert_into_leaf(&mut node, 0, key1);
        assert_eq!(node.n, 1);

        let key2 = MTKey {
            pos: MTPos::new(0, 5),
            ns: 1,
            id: 2,
            flags: 0,
            decor_data: DecorInlineData::zero(),
        };
        insert_into_leaf(&mut node, 0, key2);
        assert_eq!(node.n, 2);
        assert_eq!(node.keys[0].id, 2);
        assert_eq!(node.keys[1].id, 1);
    }
}
