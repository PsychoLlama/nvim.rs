//! Native Rust B-tree node types for marktree.
//!
//! This module defines the core data structures for the marktree B-tree:
//! - `MTNode`: A B-tree node containing keys and optional children
//! - `MarkTree`: The main tree container owning the root
//! - `MarkTreeIter`: Iterator state for tree traversal
//!
//! These types mirror the C definitions in `marktree_defs.h` but use native
//! Rust ownership and memory management.

// We use i32 for n to match C, but n is always non-negative in the B-tree context
#![allow(clippy::cast_sign_loss)]

use std::collections::HashMap;
use std::ptr::NonNull;

use crate::intersection::Intersection;
use crate::{MTKey, MTPos, MT_BRANCH_FACTOR, MT_MAX_DEPTH};

// ============================================================================
// Constants
// ============================================================================

/// Maximum number of keys per node (2 * MT_BRANCH_FACTOR - 1).
pub const MT_MAX_KEYS: usize = 2 * MT_BRANCH_FACTOR - 1;

/// Maximum number of children per internal node (2 * MT_BRANCH_FACTOR).
pub const MT_MAX_CHILDREN: usize = 2 * MT_BRANCH_FACTOR;

/// Number of meta indices tracked per child.
pub const MT_META_COUNT: usize = 5;

// ============================================================================
// Meta Index Enum
// ============================================================================

/// Meta indices for filtered iteration.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetaIndex {
    /// Inline decorations.
    Inline = 0,
    /// Line decorations.
    Lines = 1,
    /// Sign highlight decorations.
    SignHL = 2,
    /// Sign text decorations.
    SignText = 3,
    /// Concealed lines decorations.
    ConcealLines = 4,
}

impl MetaIndex {
    /// Get the count (sentinel value).
    #[must_use]
    pub const fn count() -> usize {
        MT_META_COUNT
    }
}

// ============================================================================
// MTNode - B-tree Node
// ============================================================================

/// A B-tree node in the marktree.
///
/// Nodes can be either internal (with children) or leaf (no children).
/// Keys are stored in sorted order within each node.
///
/// # Memory Layout
///
/// - Leaf nodes contain only `keys` (up to `MT_MAX_KEYS`)
/// - Internal nodes also have `children` and `meta` counts
///
/// # Invariants
///
/// - `n` is the number of valid keys (0 <= n <= MT_MAX_KEYS)
/// - `level` is 0 for leaves, >0 for internal nodes
/// - Internal nodes have `n + 1` children
/// - Keys are sorted within each node
pub struct MTNode {
    /// Number of keys in this node.
    pub n: i32,

    /// Level of this node (0 = leaf, >0 = internal).
    pub level: i16,

    /// Index of this node in parent's children array.
    pub p_idx: i16,

    /// Intersection list - tracks paired marks spanning this subtree.
    pub intersect: Intersection,

    /// Parent node (None for root).
    pub parent: Option<NonNull<MTNode>>,

    /// Keys stored in this node.
    pub keys: [MTKey; MT_MAX_KEYS],

    /// Children of this node (only valid for internal nodes).
    /// Children are heap-allocated and owned by this node.
    pub children: Option<Box<MTNodeChildren>>,
}

/// Children and metadata for internal nodes.
///
/// Separated from `MTNode` to avoid allocating this for leaf nodes.
pub struct MTNodeChildren {
    /// Child pointers (owned).
    pub ptr: [Option<Box<MTNode>>; MT_MAX_CHILDREN],

    /// Meta counts for each child subtree.
    pub meta: [[u32; MT_META_COUNT]; MT_MAX_CHILDREN],
}

impl Default for MTNodeChildren {
    fn default() -> Self {
        Self::new()
    }
}

impl MTNodeChildren {
    /// Create a new children structure with all null pointers.
    #[must_use]
    pub fn new() -> Self {
        Self {
            ptr: Default::default(),
            meta: [[0; MT_META_COUNT]; MT_MAX_CHILDREN],
        }
    }
}

impl MTNode {
    /// Create a new leaf node.
    #[must_use]
    pub fn new_leaf() -> Self {
        Self {
            n: 0,
            level: 0,
            p_idx: 0,
            intersect: Intersection::new(),
            parent: None,
            keys: [MTKey::invalid(); MT_MAX_KEYS],
            children: None,
        }
    }

    /// Create a new internal node at the given level.
    #[must_use]
    pub fn new_internal(level: i16) -> Self {
        debug_assert!(level > 0, "internal nodes must have level > 0");
        Self {
            n: 0,
            level,
            p_idx: 0,
            intersect: Intersection::new(),
            parent: None,
            keys: [MTKey::invalid(); MT_MAX_KEYS],
            children: Some(Box::new(MTNodeChildren::new())),
        }
    }

    /// Check if this is a leaf node.
    #[inline]
    #[must_use]
    pub const fn is_leaf(&self) -> bool {
        self.level == 0
    }

    /// Check if this is the root node.
    #[inline]
    #[must_use]
    pub const fn is_root(&self) -> bool {
        self.parent.is_none()
    }

    /// Check if this node is full.
    #[inline]
    #[must_use]
    pub const fn is_full(&self) -> bool {
        self.n as usize >= MT_MAX_KEYS
    }

    /// Check if this node has the minimum number of keys.
    #[inline]
    #[must_use]
    pub const fn is_minimal(&self) -> bool {
        (self.n as usize) < MT_BRANCH_FACTOR
    }

    /// Get the number of keys.
    #[inline]
    #[must_use]
    pub const fn key_count(&self) -> usize {
        self.n as usize
    }

    /// Get a key by index.
    #[inline]
    #[must_use]
    pub fn get_key(&self, idx: usize) -> &MTKey {
        debug_assert!(idx < self.n as usize, "key index out of bounds");
        &self.keys[idx]
    }

    /// Get a mutable key by index.
    #[inline]
    pub fn get_key_mut(&mut self, idx: usize) -> &mut MTKey {
        debug_assert!(idx < self.n as usize, "key index out of bounds");
        &mut self.keys[idx]
    }

    /// Set a key at the given index.
    #[inline]
    pub fn set_key(&mut self, idx: usize, key: MTKey) {
        debug_assert!(idx < MT_MAX_KEYS, "key index out of bounds");
        self.keys[idx] = key;
    }

    /// Get a child by index (for internal nodes).
    ///
    /// # Panics
    ///
    /// Panics if called on a leaf node or index is out of bounds.
    #[inline]
    #[must_use]
    pub fn get_child(&self, idx: usize) -> Option<&Self> {
        debug_assert!(idx <= self.n as usize, "child index out of bounds");
        self.children.as_ref().and_then(|c| c.ptr[idx].as_deref())
    }

    /// Get a mutable child by index (for internal nodes).
    #[inline]
    pub fn get_child_mut(&mut self, idx: usize) -> Option<&mut Self> {
        debug_assert!(idx <= self.n as usize, "child index out of bounds");
        self.children
            .as_mut()
            .and_then(|c| c.ptr[idx].as_deref_mut())
    }

    /// Take a child from this node (removes ownership).
    #[inline]
    pub fn take_child(&mut self, idx: usize) -> Option<Box<Self>> {
        self.children.as_mut().and_then(|c| c.ptr[idx].take())
    }

    /// Set a child at the given index.
    #[inline]
    pub fn set_child(&mut self, idx: usize, child: Option<Box<Self>>) {
        if let Some(ref mut c) = self.children {
            c.ptr[idx] = child;
        }
    }

    /// Get meta count for a child.
    #[inline]
    #[must_use]
    pub fn get_meta(&self, child_idx: usize, meta_idx: usize) -> u32 {
        self.children
            .as_ref()
            .map_or(0, |c| c.meta[child_idx][meta_idx])
    }

    /// Set meta count for a child.
    #[inline]
    pub fn set_meta(&mut self, child_idx: usize, meta_idx: usize, val: u32) {
        if let Some(ref mut c) = self.children {
            c.meta[child_idx][meta_idx] = val;
        }
    }

    /// Add to meta count for a child.
    #[inline]
    pub fn add_meta(&mut self, child_idx: usize, meta_idx: usize, val: u32) {
        if let Some(ref mut c) = self.children {
            c.meta[child_idx][meta_idx] += val;
        }
    }

    /// Get the meta array for a child.
    #[inline]
    #[must_use]
    pub fn get_meta_array(&self, child_idx: usize) -> Option<&[u32; MT_META_COUNT]> {
        self.children.as_ref().map(|c| &c.meta[child_idx])
    }
}

// ============================================================================
// MarkTree - Main Tree Container
// ============================================================================

/// The main marktree container.
///
/// Owns the root node and maintains metadata about the tree.
pub struct MarkTree {
    /// Root node of the tree (None if empty).
    pub root: Option<Box<MTNode>>,

    /// Aggregated meta counts for the entire tree.
    pub meta_root: [u32; MT_META_COUNT],

    /// Total number of keys in the tree.
    pub n_keys: usize,

    /// Total number of nodes in the tree.
    pub n_nodes: usize,

    /// Map from mark IDs to nodes containing them.
    /// Key is the lookup ID (ns << 33 | id << 1 | end_flag).
    pub id2node: HashMap<u64, NonNull<MTNode>>,
}

impl Default for MarkTree {
    fn default() -> Self {
        Self::new()
    }
}

impl MarkTree {
    /// Create a new empty marktree.
    #[must_use]
    pub fn new() -> Self {
        Self {
            root: None,
            meta_root: [0; MT_META_COUNT],
            n_keys: 0,
            n_nodes: 0,
            id2node: HashMap::new(),
        }
    }

    /// Check if the tree is empty.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.n_keys == 0
    }

    /// Get the number of keys in the tree.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.n_keys
    }

    /// Get the root node.
    #[inline]
    #[must_use]
    pub fn root(&self) -> Option<&MTNode> {
        self.root.as_deref()
    }

    /// Get the root node mutably.
    #[inline]
    pub fn root_mut(&mut self) -> Option<&mut MTNode> {
        self.root.as_deref_mut()
    }

    /// Get the level of the tree (0 for empty, 1 for single leaf, etc.).
    #[inline]
    #[must_use]
    pub fn level(&self) -> i16 {
        self.root.as_ref().map_or(0, |r| r.level + 1)
    }

    /// Register a key's location in the id2node map.
    pub fn register_key(&mut self, key: &MTKey, node: NonNull<MTNode>) {
        let id = crate::mt_lookup_key(key);
        self.id2node.insert(id, node);
    }

    /// Unregister a key from the id2node map.
    pub fn unregister_key(&mut self, key: &MTKey) {
        let id = crate::mt_lookup_key(key);
        self.id2node.remove(&id);
    }

    /// Look up a node by mark ID.
    #[must_use]
    pub fn lookup_node(&self, id: u64) -> Option<NonNull<MTNode>> {
        self.id2node.get(&id).copied()
    }

    // ========================================================================
    // Memory Management
    // ========================================================================

    /// Get approximate memory usage of the tree in bytes.
    ///
    /// This includes:
    /// - Size of MarkTree struct
    /// - Size of all MTNode structs
    /// - Size of MTNodeChildren for internal nodes
    /// - HashMap overhead for id2node
    #[must_use]
    pub fn memory_usage(&self) -> usize {
        let base_size = std::mem::size_of::<Self>();
        let node_size = std::mem::size_of::<MTNode>();
        let children_size = std::mem::size_of::<MTNodeChildren>();

        // Count internal vs leaf nodes
        let internal_nodes = self.count_internal_nodes();
        let leaf_nodes = self.n_nodes.saturating_sub(internal_nodes);

        // Node memory: leaf nodes have just MTNode, internal have MTNode + MTNodeChildren
        let node_memory = (leaf_nodes * node_size) + (internal_nodes * (node_size + children_size));

        // HashMap overhead (approximate)
        let hashmap_overhead = self.id2node.len()
            * (std::mem::size_of::<u64>() + std::mem::size_of::<NonNull<MTNode>>() + 8); // bucket overhead estimate

        base_size + node_memory + hashmap_overhead
    }

    /// Count internal (non-leaf) nodes in the tree.
    fn count_internal_nodes(&self) -> usize {
        fn count_internal(node: &MTNode) -> usize {
            if node.is_leaf() {
                0
            } else {
                let mut count = 1;
                if let Some(ref children) = node.children {
                    for i in 0..=node.n as usize {
                        if let Some(ref child) = children.ptr[i] {
                            count += count_internal(child);
                        }
                    }
                }
                count
            }
        }

        self.root.as_ref().map_or(0, |r| count_internal(r))
    }

    /// Get memory statistics for debugging.
    #[must_use]
    pub fn memory_stats(&self) -> MemoryStats {
        let internal_nodes = self.count_internal_nodes();
        let leaf_nodes = self.n_nodes.saturating_sub(internal_nodes);

        MemoryStats {
            total_nodes: self.n_nodes,
            internal_nodes,
            leaf_nodes,
            total_keys: self.n_keys,
            id2node_entries: self.id2node.len(),
            tree_depth: self.level() as usize,
            estimated_bytes: self.memory_usage(),
        }
    }
}

/// Memory statistics for a marktree.
#[derive(Debug, Clone, Copy)]
pub struct MemoryStats {
    /// Total number of nodes.
    pub total_nodes: usize,
    /// Number of internal (non-leaf) nodes.
    pub internal_nodes: usize,
    /// Number of leaf nodes.
    pub leaf_nodes: usize,
    /// Total number of keys.
    pub total_keys: usize,
    /// Number of entries in id2node map.
    pub id2node_entries: usize,
    /// Depth of the tree.
    pub tree_depth: usize,
    /// Estimated memory usage in bytes.
    pub estimated_bytes: usize,
}

impl Drop for MarkTree {
    fn drop(&mut self) {
        // The root is a Box, so dropping it will recursively free all nodes
        // through the children field ownership chain.
        self.root = None;
        self.id2node.clear();
    }
}

// ============================================================================
// MarkTreeIter - Iterator State
// ============================================================================

/// Saved state at each level during iteration.
#[derive(Debug, Clone, Copy, Default)]
pub struct IterLevelState {
    /// Saved oldcol for restoring position.
    pub oldcol: i32,
    /// Saved index in the node.
    pub i: i32,
}

/// Iterator for traversing the marktree.
///
/// Maintains a path from root to the current position, allowing
/// efficient forward and backward traversal.
pub struct MarkTreeIter {
    /// Current absolute position.
    pub pos: MTPos,

    /// Current level in the tree (0 = at root).
    pub lvl: i32,

    /// Current node (None if invalid/at end).
    pub x: Option<NonNull<MTNode>>,

    /// Current index within the node.
    pub i: i32,

    /// Saved state at each level for traversal.
    pub s: [IterLevelState; MT_MAX_DEPTH],

    /// Index into intersection list for overlap queries.
    pub intersect_idx: usize,

    /// Position for intersection tracking.
    pub intersect_pos: MTPos,

    /// Position within current node for intersection tracking.
    pub intersect_pos_x: MTPos,
}

impl Default for MarkTreeIter {
    fn default() -> Self {
        Self::new()
    }
}

impl MarkTreeIter {
    /// Create a new invalid iterator.
    #[must_use]
    pub fn new() -> Self {
        Self {
            pos: MTPos::new(0, 0),
            lvl: 0,
            x: None,
            i: 0,
            s: [IterLevelState::default(); MT_MAX_DEPTH],
            intersect_idx: 0,
            intersect_pos: MTPos::new(0, 0),
            intersect_pos_x: MTPos::new(0, 0),
        }
    }

    /// Check if the iterator is valid (pointing to a key).
    #[inline]
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.x.is_some()
    }

    /// Invalidate the iterator.
    #[inline]
    pub fn invalidate(&mut self) {
        self.x = None;
    }

    /// Get the current node.
    #[inline]
    #[must_use]
    pub fn node(&self) -> Option<&MTNode> {
        // SAFETY: The iterator maintains valid node references
        self.x.map(|ptr| unsafe { ptr.as_ref() })
    }

    /// Get the current node mutably.
    #[inline]
    pub fn node_mut(&mut self) -> Option<&mut MTNode> {
        // SAFETY: The iterator maintains valid node references
        self.x.map(|mut ptr| unsafe { ptr.as_mut() })
    }

    /// Get the current key.
    #[inline]
    #[must_use]
    pub fn current_key(&self) -> Option<MTKey> {
        self.node().and_then(|node| {
            if self.i >= 0 && self.i < node.n {
                Some(node.keys[self.i as usize])
            } else {
                None
            }
        })
    }

    /// Get saved state at a level.
    #[inline]
    #[must_use]
    pub fn get_state(&self, lvl: usize) -> &IterLevelState {
        debug_assert!(lvl < MT_MAX_DEPTH);
        &self.s[lvl]
    }

    /// Get mutable saved state at a level.
    #[inline]
    pub fn get_state_mut(&mut self, lvl: usize) -> &mut IterLevelState {
        debug_assert!(lvl < MT_MAX_DEPTH);
        &mut self.s[lvl]
    }

    /// Save current state at the current level.
    #[inline]
    pub fn save_state(&mut self) {
        let lvl = self.lvl as usize;
        if lvl < MT_MAX_DEPTH {
            self.s[lvl].i = self.i;
            self.s[lvl].oldcol = self.pos.col;
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
    fn test_mtnode_new_leaf() {
        let node = MTNode::new_leaf();
        assert_eq!(node.n, 0);
        assert_eq!(node.level, 0);
        assert!(node.is_leaf());
        assert!(node.is_root());
        assert!(!node.is_full());
        assert!(node.children.is_none());
    }

    #[test]
    fn test_mtnode_new_internal() {
        let node = MTNode::new_internal(1);
        assert_eq!(node.n, 0);
        assert_eq!(node.level, 1);
        assert!(!node.is_leaf());
        assert!(node.is_root());
        assert!(node.children.is_some());
    }

    #[test]
    fn test_marktree_new() {
        let tree = MarkTree::new();
        assert!(tree.is_empty());
        assert_eq!(tree.len(), 0);
        assert!(tree.root().is_none());
    }

    #[test]
    fn test_markiter_new() {
        let iter = MarkTreeIter::new();
        assert!(!iter.is_valid());
        assert!(iter.current_key().is_none());
    }

    #[test]
    fn test_mtnode_key_operations() {
        let mut node = MTNode::new_leaf();

        // Set some keys
        let key1 = MTKey {
            pos: MTPos::new(0, 0),
            ns: 1,
            id: 1,
            flags: 0,
            decor_data: DecorInlineData::zero(),
        };
        let key2 = MTKey {
            pos: MTPos::new(0, 5),
            ns: 1,
            id: 2,
            flags: 0,
            decor_data: DecorInlineData::zero(),
        };

        node.set_key(0, key1);
        node.set_key(1, key2);
        node.n = 2;

        assert_eq!(node.key_count(), 2);
        assert_eq!(node.get_key(0).id, 1);
        assert_eq!(node.get_key(1).id, 2);
    }

    #[test]
    fn test_mtnode_child_operations() {
        let mut parent = MTNode::new_internal(1);
        let child = Box::new(MTNode::new_leaf());

        parent.set_child(0, Some(child));
        assert!(parent.get_child(0).is_some());

        let taken = parent.take_child(0);
        assert!(taken.is_some());
        assert!(parent.get_child(0).is_none());
    }

    #[test]
    fn test_iter_state() {
        let mut iter = MarkTreeIter::new();
        iter.pos = MTPos::new(5, 10);
        iter.lvl = 2;
        iter.i = 3;
        iter.save_state();

        assert_eq!(iter.s[2].i, 3);
        assert_eq!(iter.s[2].oldcol, 10);
    }
}
