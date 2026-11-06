#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

//! The B-tree's nodes, and the slab that owns them.
//!
//! Every node of a tree lives in that tree's [`NodeSlab`] and is named by a
//! [`NodeId`], its slot there. Links go by id — a node's parent and children,
//! an iterator's current node, the `id2node` index, a splice's damage record —
//! so nothing holds a node's address, and a tree change that frees or moves a
//! node can at worst leave a stale id behind: reading through one answers a
//! wrong key or panics on a slot index, never reads freed memory.
//!
//! A node is a [`Node`]: its keys and child ids inline, and — for an internal
//! node only — a boxed block of its children's meta counts. Upstream allocated
//! both child arrays as a flexible tail past an internal node's struct; the
//! ids are small enough to carry on every node (a descent reads them with no
//! indirection), and the counts, five times their size, stay off the leaves.
//!
//! Freed slots go on a free list and are handed out again before the slab
//! grows, so a long-lived buffer's tree stays as large as its peak node count.
//!
//! Reach a node with `tree[id]` (`Index<NodeId>` on [`MarkTree`]); the
//! operations that touch two nodes at once (copying keys or children across, the
//! parent links) are methods on the tree.

use crate::narrow::count_as_int;
use crate::narrow::count_as_u8;
use core::ffi::c_int;
use core::ops::{Index, IndexMut, Range};

use crate::types::{MTKey, MTPos, MarkTree, MarkTreeIter, uint64_t};

use super::intersect::IdSet;
use super::key::{MT_BRANCH_FACTOR, MT_LOG2_BRANCH, MtFlags, key_cmp, mt_lookup_key};
use super::meta::{META_COUNT, MetaCount, meta_add, meta_add_key};

/// Sizes that are part of the node layout. Nested to keep the names out of the
/// flat cdef namespace `ffigen` builds.
mod sizes {
    use super::MT_BRANCH_FACTOR;

    /// The most keys a node can hold. A node is legal down to
    /// `MT_BRANCH_FACTOR - 1` keys, except the root, which may hold none.
    pub const MAX_KEYS: usize = 2 * MT_BRANCH_FACTOR as usize - 1;
    /// The most children an internal node can hold: one more than its keys,
    /// since every key separates two subtrees.
    pub const MAX_CHILDREN: usize = MAX_KEYS + 1;
}
pub use sizes::{MAX_CHILDREN, MAX_KEYS};

/// A node's slot in its tree's [`NodeSlab`].
///
/// Equality is slot identity — the tree's own notion of "the same node".
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct NodeId(u32);

impl NodeId {
    #[inline(always)]
    fn slot(self) -> usize {
        self.0 as usize
    }
}

/// The per-child meta counts of an internal node.
type ChildMeta = [MetaCount; MAX_CHILDREN];

/// What a vacant key slot holds. Only the slots below a node's key count are
/// ever read as keys.
const VACANT_KEY: MTKey = MTKey {
    pos: MTPos { row: 0, col: 0 },
    ns: 0,
    id: 0,
    flags: MtFlags::NONE,
    decor_data: super::key::MT_INVALID_KEY.decor_data,
};

/// One node of a marktree.
pub struct Node {
    key_count: u8,
    /// Distance from the leaves.
    level: u8,
    /// Which of its parent's children this node is.
    parent_index: u8,
    parent: Option<NodeId>,
    /// The ids of the paired marks whose ranges cover the whole of this node.
    intersection: IdSet,
    /// The subtrees, in order; only the first `key_count + 1` of an internal
    /// node mean anything, and none of a leaf's.
    children: [NodeId; MAX_CHILDREN],
    /// How many keys of each meta kind each subtree holds. `Some` exactly when
    /// `level` is non-zero.
    child_meta: Option<Box<ChildMeta>>,
    keys: [MTKey; MAX_KEYS],
}

impl Node {
    fn new(level: usize) -> Node {
        let child_meta = (level > 0).then(|| Box::new([[0; META_COUNT]; MAX_CHILDREN]));
        Node {
            key_count: 0,
            level: u8::try_from(level).expect("a marktree is only a few levels deep"),
            parent_index: 0,
            parent: None,
            intersection: IdSet::EMPTY,
            children: [NodeId(0); MAX_CHILDREN],
            child_meta,
            keys: [VACANT_KEY; MAX_KEYS],
        }
    }

    // -- header ------------------------------------------------------------

    /// How many keys the node currently holds.
    #[inline(always)]
    pub fn key_count(&self) -> usize {
        usize::from(self.key_count)
    }

    #[inline]
    pub fn set_key_count(&mut self, count: usize) {
        debug_assert!(count <= MAX_KEYS, "count <= MAX_KEYS");
        self.key_count = count_as_u8(count);
    }

    /// Distance from the leaves: zero for a leaf, one for a node whose children
    /// are leaves, and so on.
    #[inline(always)]
    pub fn level(&self) -> usize {
        usize::from(self.level)
    }

    /// Leaves hold keys and no children.
    #[inline(always)]
    pub fn is_leaf(&self) -> bool {
        self.level == 0
    }

    /// The node this one hangs off, or `None` at the root.
    #[inline(always)]
    pub fn parent(&self) -> Option<NodeId> {
        self.parent
    }

    #[inline]
    pub fn set_parent(&mut self, parent: Option<NodeId>) {
        self.parent = parent;
    }

    /// Which of its parent's children this node is. Meaningless at the root.
    #[inline(always)]
    pub fn parent_index(&self) -> usize {
        usize::from(self.parent_index)
    }

    #[inline]
    pub fn set_parent_index(&mut self, index: usize) {
        debug_assert!(index < MAX_CHILDREN, "index < MAX_CHILDREN");
        self.parent_index = count_as_u8(index);
    }

    // -- keys --------------------------------------------------------------

    /// The keys the node currently holds, in order.
    #[inline(always)]
    pub fn keys(&self) -> &[MTKey] {
        &self.keys[..self.key_count()]
    }

    /// Key slot `i`. Slots past the key count are legitimate mid-rebalance.
    #[inline(always)]
    pub fn key(&self, i: usize) -> MTKey {
        self.keys[i]
    }

    #[inline(always)]
    pub fn key_mut(&mut self, i: usize) -> &mut MTKey {
        &mut self.keys[i]
    }

    #[inline]
    pub fn set_key(&mut self, i: usize, key: MTKey) {
        self.keys[i] = key;
    }

    /// Shift `src` of the key array to start at `dest`, as the C's `memmove`
    /// did. Overlapping ranges are fine; the key count is the caller's to fix.
    #[inline]
    pub fn copy_keys_within(&mut self, src: Range<usize>, dest: usize) {
        self.keys.copy_within(src, dest);
    }

    // -- children ----------------------------------------------------------

    #[inline(always)]
    fn metas(&self) -> &ChildMeta {
        self.child_meta.as_deref().expect("a leaf has no children")
    }

    #[inline(always)]
    fn metas_mut(&mut self) -> &mut ChildMeta {
        self.child_meta
            .as_deref_mut()
            .expect("a leaf has no children")
    }

    /// The `i`th subtree. Valid for `i <= key_count()` of an internal node.
    #[inline(always)]
    pub fn child(&self, i: usize) -> NodeId {
        debug_assert!(!self.is_leaf() && i <= self.key_count(), "i <= key_count");
        self.children[i]
    }

    #[inline]
    pub fn set_child(&mut self, i: usize, child: NodeId) {
        self.children[i] = child;
    }

    /// How many keys of each meta kind the `i`th subtree holds.
    #[inline(always)]
    pub fn child_meta(&self, i: usize) -> &MetaCount {
        &self.metas()[i]
    }

    #[inline]
    pub fn child_meta_mut(&mut self, i: usize) -> &mut MetaCount {
        &mut self.metas_mut()[i]
    }

    /// Shift `src` of the children — ids and counts together — to start at
    /// `dest`. The children's own parent index is the caller's to fix, through
    /// [`MarkTree::reparent_children`].
    #[inline]
    pub fn copy_children_within(&mut self, src: Range<usize>, dest: usize) {
        self.children.copy_within(src.clone(), dest);
        self.metas_mut().copy_within(src, dest);
    }

    // -- the intersection set ----------------------------------------------

    /// The ids of the paired marks whose ranges cover the whole of this node.
    #[inline(always)]
    pub fn intersection(&self) -> &IdSet {
        &self.intersection
    }

    #[inline]
    pub fn intersection_mut(&mut self) -> &mut IdSet {
        &mut self.intersection
    }

    // -- derived -----------------------------------------------------------

    /// Recompute the node's own meta counts from its keys and its children's.
    pub fn meta(&self) -> MetaCount {
        let mut meta = [0; META_COUNT];
        for &key in self.keys() {
            meta_add_key(&mut meta, key);
        }
        if !self.is_leaf() {
            for count in &self.metas()[..=self.key_count()] {
                meta_add(&mut meta, count);
            }
        }
        meta
    }
}

/// Every node of one tree, by [`NodeId`], with the vacated slots kept for
/// reuse.
pub struct NodeSlab {
    slots: Vec<Node>,
    /// Vacated slots, most recently freed last.
    free: Vec<NodeId>,
}

impl NodeSlab {
    pub const fn new() -> NodeSlab {
        NodeSlab {
            slots: Vec::new(),
            free: Vec::new(),
        }
    }

    /// How many nodes are live.
    pub fn live(&self) -> usize {
        self.slots.len() - self.free.len()
    }

    /// A fresh empty node at `level`, in a vacated slot if there is one.
    fn alloc(&mut self, level: usize) -> NodeId {
        if let Some(id) = self.free.pop() {
            self.slots[id.slot()] = Node::new(level);
            return id;
        }
        let id = NodeId(u32::try_from(self.slots.len()).expect("fewer than 2^32 nodes"));
        self.slots.push(Node::new(level));
        id
    }

    /// Vacate `id`'s slot, giving back what its node owns.
    fn release(&mut self, id: NodeId) {
        let node = &mut self.slots[id.slot()];
        node.child_meta = None;
        node.intersection = IdSet::EMPTY;
        node.key_count = 0;
        self.free.push(id);
    }

    /// Forget every node.
    fn clear(&mut self) {
        self.slots.clear();
        self.free.clear();
    }
}

impl Default for NodeSlab {
    fn default() -> Self {
        NodeSlab::new()
    }
}

impl Index<NodeId> for MarkTree {
    type Output = Node;

    #[inline(always)]
    fn index(&self, id: NodeId) -> &Node {
        &self.nodes.slots[id.slot()]
    }
}

impl IndexMut<NodeId> for MarkTree {
    #[inline(always)]
    fn index_mut(&mut self, id: NodeId) -> &mut Node {
        &mut self.nodes.slots[id.slot()]
    }
}

impl MarkTree {
    /// Allocate an empty node at `level` (children are allocated with it for
    /// an internal one).
    pub fn alloc_node(&mut self, level: usize) -> NodeId {
        self.nodes.alloc(level)
    }

    /// Free one node, detached from the tree — nothing may name it afterwards
    /// except a stale iterator.
    pub fn free_node(&mut self, x: NodeId) {
        self.nodes.release(x);
    }

    /// Free every node at once.
    pub(crate) fn free_all_nodes(&mut self) {
        self.nodes.clear();
    }

    /// How many nodes the tree holds.
    pub fn node_count(&self) -> usize {
        self.nodes.live()
    }

    /// The level of the root, or `None` for an empty tree.
    pub fn root_level(&self) -> Option<usize> {
        self.root.map(|root| self[root].level())
    }

    /// Two distinct nodes at once.
    fn pair_mut(&mut self, a: NodeId, b: NodeId) -> [&mut Node; 2] {
        self.nodes
            .slots
            .get_disjoint_mut([a.slot(), b.slot()])
            .expect("two distinct live nodes")
    }

    /// Copy `src`'s keys in `range` into `dst` starting at `dest`.
    pub fn copy_keys_between(
        &mut self,
        dst: NodeId,
        dest: usize,
        src: NodeId,
        range: Range<usize>,
    ) {
        let [to, from] = self.pair_mut(dst, src);
        to.keys[dest..dest + range.len()].copy_from_slice(&from.keys[range]);
    }

    /// Copy `src`'s children in `range` — ids and counts — into `dst` starting
    /// at `dest`.
    pub fn copy_children_between(
        &mut self,
        dst: NodeId,
        dest: usize,
        src: NodeId,
        range: Range<usize>,
    ) {
        let [to, from] = self.pair_mut(dst, src);
        let span = dest..dest + range.len();
        to.children[span.clone()].copy_from_slice(&from.children[range.clone()]);
        to.metas_mut()[span].copy_from_slice(&from.metas()[range]);
    }

    /// Tell every child of `x` in `range` where it now sits, after the
    /// children moved.
    pub fn reparent_children(&mut self, x: NodeId, range: Range<usize>) {
        for i in range {
            let child = self[x].child(i);
            let node = &mut self[child];
            node.set_parent(Some(x));
            node.set_parent_index(i);
        }
    }

    /// Record that `x` is the node holding key `i`, so `marktree_lookup` can
    /// find it by id without a walk.
    #[inline]
    pub fn refkey(&mut self, x: NodeId, i: usize) {
        let id = mt_lookup_key(self[x].key(i));
        self.id2node.insert(id, x);
    }

    /// The node holding the key with this lookup handle.
    #[inline]
    pub fn id2node(&self, id: uint64_t) -> Option<NodeId> {
        self.id2node.get(&id).copied()
    }

    /// The key a positioned iterator stands on, for a consumer to edit in
    /// place (the external flags, the decoration).
    ///
    /// # Panics
    /// If the iterator is not positioned.
    pub fn key_mut(&mut self, iter: &MarkTreeIter) -> &mut MTKey {
        let node = iter.node.expect("a positioned iterator");
        let slot = usize::try_from(iter.index).expect("an iterator's slot is never negative");
        self[node].key_mut(slot)
    }

    /// An ordering key for "the gap just before index `i` of node `x`": the
    /// path from the root packed [`MT_LOG2_BRANCH`] bits per level, root in the
    /// high bits.
    ///
    /// The first index is shifted by the *node's own level* rather than by
    /// zero, so that indices taken at different depths are still comparable — a
    /// leaf's path occupies the low bits and an internal node's stops short of
    /// them, which is what makes "before this whole subtree" compare less than
    /// anything inside it. Two of these compare exactly as the positions they
    /// name do, which is what lets the overlap walk decide whether an
    /// intersecting pair started before the iterator without walking to it.
    /// Depth is bounded by `MT_MAX_DEPTH` and the branch by
    /// `2 * MT_BRANCH_FACTOR`, so the path fits in 64 bits with room for the
    /// "immediately before this node" index zero.
    pub fn pseudo_index(&self, x: NodeId, i: usize) -> uint64_t {
        let mut index: uint64_t = 0;
        let mut node = &self[x];
        let mut off = MT_LOG2_BRANCH as usize * node.level();
        let mut i = i;
        loop {
            index |= ((i + 1) as uint64_t) << off;
            off += MT_LOG2_BRANCH as usize;
            let Some(parent) = node.parent() else {
                return index;
            };
            i = node.parent_index();
            node = &self[parent];
        }
    }
}

/// Where key `k` belongs among `keys`, which are sorted.
///
/// Answers the index of the last key that is not greater than `k`, and whether
/// that key compares equal. An empty node answers `-1`, and so does a `k` that
/// sorts before every key present — the caller reads the result as "the gap
/// after index `n`", so `-1` means "before everything".
pub fn find_key(keys: &[MTKey], k: MTKey) -> (c_int, bool) {
    if keys.is_empty() {
        return (-1, false);
    }
    let mut begin = 0;
    let mut end = keys.len();
    while begin < end {
        let mid = (begin + end) >> 1;
        if key_cmp(keys[mid], k) < 0 {
            begin = mid + 1;
        } else {
            end = mid;
        }
    }
    if begin == keys.len() {
        return (count_as_int(keys.len()) - 1, false);
    }
    let found = key_cmp(k, keys[begin]) == 0;
    (count_as_int(begin) - c_int::from(!found), found)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoration::DECOR_HIGHLIGHT_INLINE_INIT;
    use crate::marktree::key::MtFlags;
    use crate::types::{DecorInline, MTPos};

    fn key(row: i32, col: i32, flags: MtFlags) -> MTKey {
        MTKey {
            pos: MTPos { row, col },
            ns: 0,
            id: 0,
            flags,
            decor_data: DecorInline::Hl(DECOR_HIGHLIGHT_INLINE_INIT),
        }
    }

    #[test]
    fn a_leaf_has_no_children_and_an_internal_node_has_all_of_them() {
        assert!(Node::new(0).child_meta.is_none());
        let node = Node::new(1);
        assert_eq!(node.metas().len(), MAX_CHILDREN);
        assert_eq!(node.keys.len(), MAX_KEYS);
    }

    #[test]
    fn a_freed_slot_is_handed_out_again_before_the_slab_grows() {
        let mut slab = NodeSlab::new();
        let a = slab.alloc(0);
        let b = slab.alloc(1);
        assert_eq!(slab.live(), 2);
        slab.release(a);
        assert_eq!(slab.live(), 1);
        let c = slab.alloc(2);
        assert_eq!(c, a);
        assert_eq!(slab.slots.len(), 2);
        assert_eq!(slab.slots[c.slot()].level(), 2);
        assert!(slab.slots[c.slot()].child_meta.is_some());
        assert_ne!(b, c);
        slab.clear();
        assert_eq!(slab.live(), 0);
    }

    #[test]
    fn an_empty_node_answers_before_everything() {
        assert_eq!(find_key(&[], key(0, 0, MtFlags::REAL)), (-1, false));
    }

    #[test]
    fn finds_an_exact_match_and_reports_it() {
        let keys = [
            key(1, 0, MtFlags::REAL),
            key(1, 5, MtFlags::REAL),
            key(3, 2, MtFlags::REAL),
        ];
        assert_eq!(find_key(&keys, key(1, 5, MtFlags::REAL)), (1, true));
        assert_eq!(find_key(&keys, key(3, 2, MtFlags::REAL)), (2, true));
    }

    #[test]
    fn a_key_between_two_others_lands_on_the_earlier_one() {
        let keys = [
            key(1, 0, MtFlags::REAL),
            key(1, 5, MtFlags::REAL),
            key(3, 2, MtFlags::REAL),
        ];
        assert_eq!(find_key(&keys, key(1, 3, MtFlags::REAL)), (0, false));
        assert_eq!(find_key(&keys, key(9, 9, MtFlags::REAL)), (2, false));
        // Before the first key at all.
        assert_eq!(find_key(&keys, key(0, 0, MtFlags::REAL)), (-1, false));
    }

    #[test]
    fn gravity_breaks_a_tie_the_way_the_order_does() {
        let keys = [
            key(1, 1, MtFlags::REAL),
            key(1, 1, MtFlags::REAL | MtFlags::RIGHT_GRAVITY),
        ];
        assert_eq!(
            find_key(&keys, key(1, 1, MtFlags::REAL | MtFlags::RIGHT_GRAVITY)),
            (1, true)
        );
        assert_eq!(find_key(&keys, key(1, 1, MtFlags::REAL)), (0, true));
    }
}
