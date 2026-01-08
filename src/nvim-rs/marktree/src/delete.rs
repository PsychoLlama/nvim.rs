//! Native Rust B-tree deletion operations for marktree.
//!
//! This module implements deletion with node rebalancing (pivoting and merging)
//! using native Rust types from the `node` module.

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
// Allow if-then patterns for clarity in complex tree operations
#![allow(clippy::useless_let_if_seq)]
// Allow range+1 for clarity in some loop bounds
#![allow(clippy::range_plus_one)]

use std::ptr::NonNull;

use crate::insert::meta_describe_key;
use crate::node::{MTNode, MarkTree, MarkTreeIter, MT_META_COUNT};
use crate::{mt_lookup_key, relative, unrelative, MTPos, MT_BRANCH_FACTOR};

#[cfg(test)]
use crate::MTKey;

/// Minimum number of keys in a non-root node.
const T: usize = MT_BRANCH_FACTOR;

// ============================================================================
// Pivoting Operations
// ============================================================================

/// Pivot right: steal a key from the left sibling.
///
/// Takes the last key from left sibling, moves the separator key down to right,
/// and promotes the stolen key to be the new separator.
///
/// # Arguments
/// * `tree` - The tree being modified
/// * `p` - Parent node containing both siblings
/// * `i` - Index of the left sibling (right sibling is at i+1)
fn pivot_right(tree: &mut MarkTree, p: &mut MTNode, i: usize) {
    // Get pointers we need before mutable borrowing
    let p_ptr = unsafe { NonNull::new_unchecked(p as *mut MTNode) };

    let children = p.children.as_mut().unwrap();

    // Get left sibling info
    let x = children.ptr[i].as_mut().unwrap();
    let x_n = x.n as usize;
    let stolen_key = x.keys[x_n - 1];
    x.n -= 1;

    // Store the separator key from parent
    let separator_key = p.keys[i];

    // Meta tracking
    let mut meta_inc_y = [0u32; MT_META_COUNT];
    meta_describe_key(&separator_key, &mut meta_inc_y);
    let mut meta_inc_x = [0u32; MT_META_COUNT];
    meta_describe_key(&stolen_key, &mut meta_inc_x);

    // Handle internal node case - move child from x to y
    let is_leaf = children.ptr[i].as_ref().unwrap().is_leaf();
    let mut stolen_child: Option<Box<MTNode>> = None;
    let mut stolen_meta = [0u32; MT_META_COUNT];

    if !is_leaf {
        let x = children.ptr[i].as_mut().unwrap();
        let x_children = x.children.as_mut().unwrap();
        stolen_child = x_children.ptr[x.n as usize].take();
        stolen_meta = x_children.meta[x.n as usize];
    }

    // Get right sibling and shift its keys right
    let y = children.ptr[i + 1].as_mut().unwrap();
    let y_n = y.n as usize;
    let y_ptr = unsafe { NonNull::new_unchecked(y.as_mut() as *mut MTNode) };

    // Shift keys right in y
    for j in (0..y_n).rev() {
        y.keys[j + 1] = y.keys[j];
    }

    // If internal node, shift children and meta right
    if !is_leaf {
        let y_children = y.children.as_mut().unwrap();
        for j in (0..=y_n).rev() {
            y_children.ptr[j + 1] = y_children.ptr[j].take();
            y_children.meta[j + 1] = y_children.meta[j];
        }

        // Update p_idx for shifted children
        for j in 1..y_n + 2 {
            if let Some(ref mut c) = y_children.ptr[j] {
                c.p_idx = j as i16;
            }
        }

        // Insert stolen child at position 0
        if let Some(mut child) = stolen_child {
            child.parent = Some(y_ptr);
            child.p_idx = 0;
            y_children.ptr[0] = Some(child);
            y_children.meta[0] = stolen_meta;

            // Update meta
            for m in 0..MT_META_COUNT {
                children.meta[i + 1][m] += stolen_meta[m];
                children.meta[i][m] -= stolen_meta[m];
            }
        }
    }

    // Move separator key down to y
    let y = children.ptr[i + 1].as_mut().unwrap();
    y.keys[0] = separator_key;

    // Update the parent's separator key
    p.keys[i] = stolen_key;

    // Adjust positions for relative encoding
    if i > 0 {
        unrelative(p.keys[i - 1].pos, &mut p.keys[i].pos);
    }

    // Make first key of y relative to new separator
    relative(p.keys[i].pos, &mut y.keys[0].pos);

    // Adjust remaining keys in y relative to new first key
    for k in 1..y.n as usize + 1 {
        unrelative(y.keys[0].pos, &mut y.keys[k].pos);
    }

    y.n += 1;

    // Update meta counts
    for m in 0..MT_META_COUNT {
        children.meta[i + 1][m] += meta_inc_y[m];
        children.meta[i][m] -= meta_inc_x[m];
    }

    // Register the moved keys in id2node
    let y = children.ptr[i + 1].as_ref().unwrap();
    let y_ptr = unsafe { NonNull::new_unchecked(y.as_ref() as *const MTNode as *mut MTNode) };
    tree.register_key(&y.keys[0], y_ptr);

    tree.register_key(&p.keys[i], p_ptr);
}

/// Pivot left: steal a key from the right sibling.
///
/// Takes the first key from right sibling, moves the separator key down to left,
/// and promotes the stolen key to be the new separator.
fn pivot_left(tree: &mut MarkTree, p: &mut MTNode, i: usize) {
    // Get pointers we need before mutable borrowing
    let p_ptr = unsafe { NonNull::new_unchecked(p as *mut MTNode) };

    let children = p.children.as_mut().unwrap();

    // Store original values
    let y = children.ptr[i + 1].as_mut().unwrap();
    let y_n = y.n as usize;
    let is_leaf = y.is_leaf();

    // Reverse the relative encoding in y before stealing
    for k in 1..y_n {
        relative(y.keys[0].pos, &mut y.keys[k].pos);
    }
    unrelative(p.keys[i].pos, &mut y.keys[0].pos);

    // Store the key we're stealing (first key of right sibling)
    let stolen_key = y.keys[0];

    // Store the separator key
    let separator_key = p.keys[i];

    // Meta tracking
    let mut meta_inc_x = [0u32; MT_META_COUNT];
    meta_describe_key(&separator_key, &mut meta_inc_x);
    let mut meta_inc_y = [0u32; MT_META_COUNT];
    meta_describe_key(&stolen_key, &mut meta_inc_y);

    // Shift keys left in y
    for j in 0..y_n - 1 {
        y.keys[j] = y.keys[j + 1];
    }
    y.n -= 1;

    // If internal node, shift children and meta left
    let mut stolen_child: Option<Box<MTNode>> = None;
    let mut stolen_meta = [0u32; MT_META_COUNT];

    if !is_leaf {
        let y_children = y.children.as_mut().unwrap();

        // Save the first child to move to x
        stolen_child = y_children.ptr[0].take();
        stolen_meta = y_children.meta[0];

        // Shift children left
        for j in 0..y_n {
            y_children.ptr[j] = y_children.ptr[j + 1].take();
            y_children.meta[j] = y_children.meta[j + 1];
        }

        // Update p_idx for shifted children
        for j in 0..y_n {
            if let Some(ref mut c) = y_children.ptr[j] {
                c.p_idx = j as i16;
            }
        }
    }

    // Make the remaining keys in y relative to new first key
    let y = children.ptr[i + 1].as_mut().unwrap();
    if y.n > 0 {
        // Restore relative encoding
        if i > 0 {
            relative(p.keys[i - 1].pos, &mut p.keys[i].pos);
        }
        relative(p.keys[i].pos, &mut y.keys[0].pos);

        for k in 1..y.n as usize {
            unrelative(y.keys[0].pos, &mut y.keys[k].pos);
        }
    }

    // Add separator key to end of x
    let x = children.ptr[i].as_mut().unwrap();
    let x_n = x.n as usize;
    let x_ptr = unsafe { NonNull::new_unchecked(x.as_mut() as *mut MTNode) };
    x.keys[x_n] = separator_key;
    x.n += 1;

    // If internal, add the stolen child
    if let Some(mut child) = stolen_child {
        child.parent = Some(x_ptr);
        child.p_idx = x_n as i16 + 1;

        let x_children = x.children.as_mut().unwrap();
        x_children.ptr[x_n + 1] = Some(child);
        x_children.meta[x_n + 1] = stolen_meta;

        // Update meta for the moved child
        for m in 0..MT_META_COUNT {
            children.meta[i][m] += stolen_meta[m];
            children.meta[i + 1][m] -= stolen_meta[m];
        }
    }

    // Update parent's separator key
    p.keys[i] = stolen_key;

    // Update meta counts
    for m in 0..MT_META_COUNT {
        children.meta[i][m] += meta_inc_x[m];
        children.meta[i + 1][m] -= meta_inc_y[m];
    }

    // Register the moved keys in id2node
    let x = children.ptr[i].as_ref().unwrap();
    let x_ptr = unsafe { NonNull::new_unchecked(x.as_ref() as *const MTNode as *mut MTNode) };
    tree.register_key(&x.keys[x.n as usize - 1], x_ptr);

    tree.register_key(&p.keys[i], p_ptr);
}

// ============================================================================
// Merge Operations
// ============================================================================

/// Merge two sibling nodes.
///
/// Merges child at index i with child at index i+1, using the separator key
/// at index i. The result is stored in the left child, and the right child
/// is removed.
fn merge_nodes(tree: &mut MarkTree, p: &mut MTNode, i: usize) {
    let children = p.children.as_mut().unwrap();

    // Get the separator key
    let mut separator_key = p.keys[i];
    if i > 0 {
        relative(p.keys[i - 1].pos, &mut separator_key.pos);
    }

    // Meta for separator
    let mut meta_inc = [0u32; MT_META_COUNT];
    meta_describe_key(&separator_key, &mut meta_inc);

    // Take ownership of the right child
    let mut right = children.ptr[i + 1].take().unwrap();
    let right_n = right.n as usize;

    // Get mutable reference to left child
    let left = children.ptr[i].as_mut().unwrap();
    let left_n = left.n as usize;

    // Add separator key to left
    left.keys[left_n] = separator_key;
    let left_ptr = unsafe { NonNull::new_unchecked(left.as_mut() as *mut MTNode) };
    tree.register_key(&separator_key, left_ptr);

    // Copy keys from right to left
    for j in 0..right_n {
        let mut key = right.keys[j];
        unrelative(separator_key.pos, &mut key.pos);
        left.keys[left_n + 1 + j] = key;
        tree.register_key(&key, left_ptr);
    }

    // If internal node, copy children
    if !left.is_leaf() {
        let right_children = right.children.as_mut().unwrap();
        let left_children = left.children.as_mut().unwrap();

        for j in 0..=right_n {
            let mut child = right_children.ptr[j].take().unwrap();
            child.parent = Some(left_ptr);
            child.p_idx = (left_n + 1 + j) as i16;

            left_children.meta[left_n + 1 + j] = right_children.meta[j];
            left_children.ptr[left_n + 1 + j] = Some(child);
        }
    }

    // Update left's key count
    left.n = (left_n + 1 + right_n) as i32;

    // Merge intersect lists
    for idx in 0..right.intersect.len() {
        if let Some(id) = right.intersect.get(idx) {
            if !left.intersect.has(id) {
                left.intersect.insert(id);
            }
        }
    }

    // Update meta for left child
    for m in 0..MT_META_COUNT {
        children.meta[i][m] += children.meta[i + 1][m] + meta_inc[m];
    }

    // Remove separator key and right child from parent
    let p_n = p.n as usize;
    for j in i..p_n - 1 {
        p.keys[j] = p.keys[j + 1];
    }
    for j in i + 1..p_n {
        children.ptr[j] = children.ptr[j + 1].take();
        children.meta[j] = children.meta[j + 1];
    }

    // Update p_idx for shifted children
    for j in i + 1..p_n {
        if let Some(ref mut c) = children.ptr[j] {
            c.p_idx = j as i16;
        }
    }

    p.n -= 1;

    // Free the right node
    tree.n_nodes -= 1;
    drop(right);
}

// ============================================================================
// Deletion from Leaf
// ============================================================================

/// Delete key at position i from a leaf node.
fn delete_from_leaf(node: &mut MTNode, i: usize) {
    debug_assert!(node.is_leaf());

    let n = node.n as usize;
    // Shift keys left
    for j in i..n - 1 {
        node.keys[j] = node.keys[j + 1];
    }
    node.n -= 1;
}

// ============================================================================
// Tree Deletion
// ============================================================================

impl MarkTree {
    /// Delete key at iterator position.
    ///
    /// The iterator is left pointing at the key after the deleted one.
    /// Returns the ID of the paired end mark if this was a paired start mark.
    pub fn del_itr(&mut self, itr: &mut MarkTreeIter) -> Option<u64> {
        let x_ptr = itr.x?;
        let i = itr.i as usize;

        let node = unsafe { x_ptr.as_ref() };
        if i >= node.n as usize {
            return None;
        }

        let key = node.keys[i];
        let id = mt_lookup_key(&key);

        // Track meta changes
        let mut meta_inc = [0u32; MT_META_COUNT];
        meta_describe_key(&key, &mut meta_inc);

        // Handle paired marks
        let other_id = if key.flags & crate::flags::MT_FLAG_PAIRED != 0
            && key.flags & crate::flags::MT_FLAG_ORPHANED == 0
        {
            let end_flag = if key.flags & crate::flags::MT_FLAG_END != 0 {
                0
            } else {
                crate::MARKTREE_END_FLAG
            };
            Some(id ^ end_flag)
        } else {
            None
        };

        // If at internal node, steal predecessor
        if !node.is_leaf() {
            // Find predecessor (rightmost key in left subtree)
            itr.prev();
            return self.del_itr(itr);
        }

        // Delete from leaf
        let node = unsafe { x_ptr.as_ptr().as_mut().unwrap() };
        delete_from_leaf(node, i);

        // Unregister from id2node
        self.id2node.remove(&id);
        self.n_keys -= 1;

        // Update meta counts up to root
        let mut current = x_ptr;
        while let Some(parent_ptr) = unsafe { current.as_ref() }.parent {
            let current_node = unsafe { current.as_ref() };
            let p_idx = current_node.p_idx as usize;

            // We need mutable access to update meta
            let parent_mut = unsafe { parent_ptr.as_ptr().as_mut().unwrap() };
            let children_mut = parent_mut.children.as_mut().unwrap();

            for m in 0..MT_META_COUNT {
                children_mut.meta[p_idx][m] =
                    children_mut.meta[p_idx][m].saturating_sub(meta_inc[m]);
            }

            current = parent_ptr;
        }

        // Update meta_root
        for m in 0..MT_META_COUNT {
            self.meta_root[m] = self.meta_root[m].saturating_sub(meta_inc[m]);
        }

        // Rebalance if needed
        self.rebalance_after_delete(itr, x_ptr);

        // Fix iterator position
        if itr.x.is_some() {
            let node = unsafe { itr.x.unwrap().as_ref() };
            if itr.i >= node.n {
                itr.next();
            }
        }

        other_id
    }

    /// Rebalance tree after deletion.
    fn rebalance_after_delete(&mut self, itr: &mut MarkTreeIter, mut x_ptr: NonNull<MTNode>) {
        let mut rlvl = itr.lvl - 1;

        loop {
            let x = unsafe { x_ptr.as_ref() };

            // Check if this is root
            if x.parent.is_none() {
                break;
            }

            // Check if node has enough keys
            if x.n as usize >= T - 1 {
                break;
            }

            let parent_ptr = x.parent.unwrap();
            let pi = x.p_idx as usize;

            // Get parent and siblings
            let parent = unsafe { parent_ptr.as_ptr().as_mut().unwrap() };
            let children = parent.children.as_ref().unwrap();

            // Try to steal from left sibling
            if pi > 0 {
                let left = children.ptr[pi - 1].as_ref().unwrap();
                if left.n as usize > T - 1 {
                    pivot_right(self, parent, pi - 1);
                    // Update iterator
                    if rlvl >= 0 {
                        itr.s[rlvl as usize].i += 1;
                    } else {
                        itr.i += 1;
                    }
                    break;
                }
            }

            // Try to steal from right sibling
            if pi < parent.n as usize {
                let right = children.ptr[pi + 1].as_ref().unwrap();
                if right.n as usize > T - 1 {
                    pivot_left(self, parent, pi);
                    break;
                }
            }

            // Must merge
            if pi > 0 {
                // Merge with left sibling
                merge_nodes(self, parent, pi - 1);

                // Update iterator indices
                if rlvl >= 0 {
                    itr.s[rlvl as usize].i += T as i32;
                } else {
                    itr.i += T as i32;
                }

                // Iterator now points to merged node
                let merged = parent.children.as_ref().unwrap().ptr[pi - 1]
                    .as_ref()
                    .unwrap();
                x_ptr = unsafe {
                    NonNull::new_unchecked(merged.as_ref() as *const MTNode as *mut MTNode)
                };
                itr.x = Some(x_ptr);

                if rlvl >= 0 {
                    itr.s[rlvl as usize].i -= 1;
                }
            } else {
                // Merge with right sibling
                merge_nodes(self, parent, pi);
                // x is already the left (merged) node, no iterator adjustment needed
            }

            rlvl -= 1;
            x_ptr = parent_ptr;
        }

        // Handle root becoming empty
        if let Some(ref root) = self.root {
            if root.n == 0 && !root.is_leaf() {
                // Root has no keys but has a child
                let mut old_root = self.root.take().unwrap();
                let children = old_root.children.as_mut().unwrap();
                let mut new_root = children.ptr[0].take().unwrap();
                new_root.parent = None;

                // Update iterator
                if itr.lvl > 0 {
                    for i in 0..itr.lvl as usize - 1 {
                        itr.s[i] = itr.s[i + 1];
                    }
                    itr.lvl -= 1;
                }

                self.root = Some(new_root);
                self.n_nodes -= 1;
            }
        }
    }

    /// Delete a specific mark by ID.
    ///
    /// Returns true if the mark was found and deleted.
    pub fn delete(&mut self, id: u64) -> bool {
        let mut itr = MarkTreeIter::new();
        if self.lookup_id(&mut itr, id) {
            self.del_itr(&mut itr);
            true
        } else {
            false
        }
    }

    /// Delete a paired mark (both start and end).
    pub fn del_pair(&mut self, id: u64) -> bool {
        // Delete start
        if !self.delete(id) {
            return false;
        }

        // Delete end
        let end_id = id | crate::MARKTREE_END_FLAG;
        self.delete(end_id);

        true
    }

    /// Clear all marks from the tree.
    pub fn clear(&mut self) {
        self.root = None;
        self.id2node.clear();
        self.n_keys = 0;
        self.n_nodes = 0;
        self.meta_root = [0; MT_META_COUNT];
    }

    /// Look up a mark by ID and position iterator at it.
    fn lookup_id(&self, itr: &mut MarkTreeIter, id: u64) -> bool {
        if let Some(&node_ptr) = self.id2node.get(&id) {
            let node = unsafe { node_ptr.as_ref() };

            // Find the key in this node
            for i in 0..node.n as usize {
                if mt_lookup_key(&node.keys[i]) == id {
                    itr.x = Some(node_ptr);
                    itr.i = i as i32;

                    // Build path to root for iterator
                    let mut current = node_ptr;
                    let mut lvl = 0;
                    while let Some(parent) = unsafe { current.as_ref() }.parent {
                        if lvl < itr.s.len() {
                            itr.s[lvl].i = i32::from(unsafe { current.as_ref() }.p_idx);
                        }
                        lvl += 1;
                        current = parent;
                    }
                    itr.lvl = lvl as i32;

                    // Calculate position
                    itr.pos = MTPos::new(0, 0);
                    self.calculate_absolute_pos(itr);

                    return true;
                }
            }
        }
        false
    }

    /// Calculate absolute position for iterator.
    #[allow(clippy::unused_self)]
    fn calculate_absolute_pos(&self, itr: &mut MarkTreeIter) {
        let Some(x_ptr) = itr.x else { return };
        let node = unsafe { x_ptr.as_ref() };

        // Get the key's position
        let i = itr.i as usize;
        if i >= node.n as usize {
            return;
        }

        let mut pos = node.keys[i].pos;

        // Walk up to root, unrelativizing
        let mut current = x_ptr;
        while let Some(parent_ptr) = unsafe { current.as_ref() }.parent {
            let current_node = unsafe { current.as_ref() };
            let p_idx = current_node.p_idx as usize;
            let parent = unsafe { parent_ptr.as_ref() };

            if p_idx > 0 {
                unrelative(parent.keys[p_idx - 1].pos, &mut pos);
            }

            current = parent_ptr;
        }

        itr.pos = pos;
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flags::MT_FLAG_REAL;

    fn create_key(row: i32, col: i32, id: u32) -> MTKey {
        MTKey {
            pos: MTPos::new(row, col),
            ns: 1,
            id,
            flags: MT_FLAG_REAL,
            decor_data: 0,
        }
    }

    #[test]
    fn test_delete_single() {
        let mut tree = MarkTree::new();

        let key = create_key(0, 5, 1);
        tree.put_key(key);

        assert_eq!(tree.n_keys, 1);

        let deleted = tree.delete(mt_lookup_key(&key));
        assert!(deleted);
        assert_eq!(tree.n_keys, 0);
    }

    #[test]
    fn test_delete_multiple() {
        let mut tree = MarkTree::new();

        // Insert 10 keys
        let mut keys = Vec::new();
        for id in 1..=10 {
            let key = create_key(0, id as i32 * 5, id);
            keys.push(key);
            tree.put_key(key);
        }

        assert_eq!(tree.n_keys, 10);

        // Delete middle key (id=5, index 4)
        let deleted = tree.delete(mt_lookup_key(&keys[4]));
        assert!(deleted);
        assert_eq!(tree.n_keys, 9);

        // Delete first key (id=1, index 0)
        let deleted = tree.delete(mt_lookup_key(&keys[0]));
        assert!(deleted);
        assert_eq!(tree.n_keys, 8);

        // Delete last key (id=10, index 9)
        let deleted = tree.delete(mt_lookup_key(&keys[9]));
        assert!(deleted);
        assert_eq!(tree.n_keys, 7);
    }

    #[test]
    fn test_clear() {
        let mut tree = MarkTree::new();

        for id in 1..=20 {
            let key = create_key(0, id as i32 * 5, id);
            tree.put_key(key);
        }

        assert_eq!(tree.n_keys, 20);

        tree.clear();

        assert_eq!(tree.n_keys, 0);
        assert!(tree.root.is_none());
    }

    #[test]
    fn test_delete_nonexistent() {
        let mut tree = MarkTree::new();

        let key = create_key(0, 5, 1);
        tree.put_key(key);

        // Try to delete non-existent key
        let deleted = tree.delete(999);
        assert!(!deleted);
        assert_eq!(tree.n_keys, 1);
    }

    #[test]
    fn test_delete_all() {
        let mut tree = MarkTree::new();

        // Insert keys
        let mut keys = Vec::new();
        for id in 1..=5 {
            let key = create_key(0, id as i32 * 5, id);
            keys.push(key);
            tree.put_key(key);
        }

        // Delete all
        for key in &keys {
            let deleted = tree.delete(mt_lookup_key(key));
            assert!(deleted);
        }

        assert_eq!(tree.n_keys, 0);
    }
}
