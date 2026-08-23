#![deny(unsafe_op_in_unsafe_fn)]

//! The B-tree's node representation.
//!
//! A node is one `xcalloc`'d block. Leaves are `size_of::<MTNode>()` bytes; an
//! internal node is [`ILEN`] bytes, the extra tail holding the child pointers
//! and their meta counts — `mtnode_s::s` is a flexible array member in the C
//! and stays one here, so [`inner`] is the only way to reach that tail.
//!
//! Two properties of the layout are load-bearing and constrain what this
//! module may hand out:
//!
//! * The tail lives *past* `size_of::<MTNode>()`, so a `&MTNode`/`&mut MTNode`
//!   does not cover it. Reaching the children through such a reference is out
//!   of bounds for its tag.
//! * `MTNode::intersect` is a `kvec_withinit_t`, whose `items` points at the
//!   node's own `init_array` until the set outgrows it. A `&mut MTNode`
//!   invalidates that self-pointer.
//!
//! So nodes are addressed by `*mut MTNode` throughout and never by reference.
//!
//! # Reaching a node from Rust
//!
//! [`Node`] is how the rest of the tree does that. It is a `Copy` wrapper over
//! the raw pointer whose *constructor* carries the whole validity contract:
//! once you hold a `Node`, every accessor on it is ordinary checked code. That
//! is the same inversion the other newtypes in this tree use, minus the
//! `Deref` — the two layout properties above are exactly the reasons a `Node`
//! can never hand out a `&MTNode`, so it hands out fields instead.
//!
//! The references it *does* hand out (`&mut [MTKey; MAX_KEYS]` and the child
//! tail) each cover one field and are derived afresh from the raw pointer on
//! every call, so they never overlap the intersection set's self-pointer and
//! never outlive their statement. Two of them must not be held over each other
//! for the same node; every method here derives one, uses it and drops it.

use core::ffi::c_int;
use core::ops::Range;
use core::ptr;

use crate::map::{map_put_ref_uint64_t_ptr_t, mh_get_uint64_t};
use crate::memory::{xcalloc, xfree};
use crate::types::{
    MTKey, MTNode, Map_uint64_t_ptr_t, MarkTree, mtnode_inner_s, ptr_t, uint32_t, uint64_t,
};

use super::intersect::IdSet;
use super::key::{MT_BRANCH_FACTOR, MT_LOG2_BRANCH, key_cmp, mt_lookup_key};
use super::meta::{META_COUNT, MetaCount, meta_add, meta_add_key};

/// Size of an internal node: the common header plus the child tail.
pub const ILEN: usize = size_of::<MTNode>() + size_of::<mtnode_inner_s>();

/// Sizes that are part of the node layout. Nested to keep the names out of the
/// flat cdef namespace `ffigen` builds.
mod sizes {
    use super::MT_BRANCH_FACTOR;

    /// How many ids a node's intersection set holds before it goes to the heap.
    pub const INTERSECT_INLINE: usize = 4;
    /// The most keys a node can hold. A node is legal down to
    /// `MT_BRANCH_FACTOR - 1` keys, except the root, which may hold none.
    pub const MAX_KEYS: usize = 2 * MT_BRANCH_FACTOR as usize - 1;
    /// The most children an internal node can hold: one more than its keys,
    /// since every key separates two subtrees.
    pub const MAX_CHILDREN: usize = MAX_KEYS + 1;
}
pub use sizes::{INTERSECT_INLINE, MAX_CHILDREN, MAX_KEYS};

/// One live node of a marktree.
///
/// Every accessor below is safe *because* [`Node::new`] is not: constructing
/// one is where the tree's shape is promised, and the promise covers the whole
/// tree reachable from the node, which is what lets [`Node::parent`] and
/// [`Node::child`] hand back `Node`s of their own without a second contract.
///
/// Equality is pointer identity — the tree's own notion of "the same node".
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Node(*mut MTNode);

impl Node {
    /// # Safety
    /// `x` must be non-null and name a live node — one allocated by
    /// [`marktree_alloc_node`], with [`ILEN`] bytes if its `level` is non-zero
    /// — belonging to a well-formed tree whose other nodes are live too, and it
    /// must stay live and unmoved for as long as this `Node` is used. No
    /// `&MTNode`/`&mut MTNode` to it may exist meanwhile (see the module docs).
    #[inline]
    pub unsafe fn new(x: *mut MTNode) -> Node {
        debug_assert!(!x.is_null(), "!x.is_null()");
        Node(x)
    }

    /// The same, for a pointer that is allowed to be null — a root's parent, or
    /// an empty tree's root.
    ///
    /// # Safety
    /// As [`Node::new`], for a non-null `x`.
    #[inline]
    pub unsafe fn from_ptr(x: *mut MTNode) -> Option<Node> {
        if x.is_null() { None } else { Some(Node(x)) }
    }

    /// The address the C ABI, the id map and the layout tests know it by.
    #[inline]
    pub fn as_ptr(self) -> *mut MTNode {
        self.0
    }

    // -- header ------------------------------------------------------------

    /// How many keys the node currently holds.
    #[inline]
    pub fn key_count(self) -> usize {
        // SAFETY: `self` is live, so its header is initialised (`n` is
        // non-negative for every node the tree publishes).
        unsafe { (*self.0).n as usize }
    }

    #[inline]
    pub fn set_key_count(self, count: usize) {
        debug_assert!(count <= MAX_KEYS, "count <= MAX_KEYS");
        // SAFETY: `self` is live and nothing else borrows its header.
        unsafe { (*self.0).n = count as i32 };
    }

    /// Distance from the leaves: zero for a leaf, one for a node whose children
    /// are leaves, and so on.
    #[inline]
    pub fn level(self) -> usize {
        // SAFETY: `self` is live; `level` is non-negative by construction.
        unsafe { (*self.0).level as usize }
    }

    #[inline]
    pub fn set_level(self, level: usize) {
        // SAFETY: `self` is live and nothing else borrows its header.
        unsafe { (*self.0).level = level as i16 };
    }

    /// Leaves hold keys and no children, and are allocated without the tail, so
    /// this is also "reaching [`Node::child`] on it would be out of bounds".
    #[inline]
    pub fn is_leaf(self) -> bool {
        self.level() == 0
    }

    /// The node this one hangs off, or `None` at the root.
    #[inline]
    pub fn parent(self) -> Option<Node> {
        // SAFETY: `self` is live, and its parent is live too or null — the
        // whole tree is live per `Node::new`'s contract.
        unsafe { Node::from_ptr((*self.0).parent) }
    }

    #[inline]
    pub fn set_parent(self, parent: Option<Node>) {
        let ptr = parent.map_or(ptr::null_mut(), Node::as_ptr);
        // SAFETY: `self` is live and nothing else borrows its header.
        unsafe { (*self.0).parent = ptr };
    }

    /// Which of its parent's children this node is. Meaningless at the root.
    #[inline]
    pub fn parent_index(self) -> usize {
        // SAFETY: `self` is live; `p_idx` is non-negative by construction.
        unsafe { (*self.0).p_idx as usize }
    }

    #[inline]
    pub fn set_parent_index(self, index: usize) {
        debug_assert!(index < MAX_CHILDREN, "index < MAX_CHILDREN");
        // SAFETY: `self` is live and nothing else borrows its header.
        unsafe { (*self.0).p_idx = index as i16 };
    }

    // -- keys --------------------------------------------------------------

    /// Every slot the key array has, live or not.
    ///
    /// Private: writing past `key_count` is legitimate mid-rebalance but only
    /// through the shifting helpers below, which keep the count honest.
    #[inline]
    fn key_slots<'a>(self) -> &'a mut [MTKey; MAX_KEYS] {
        // SAFETY: `self` is live, so its key array is initialised storage
        // within the allocation; the reference is derived here, used by the
        // caller's statement and dropped, so it never overlaps another.
        unsafe { &mut (*self.0).key }
    }

    /// The keys the node currently holds, in order.
    #[inline]
    pub fn keys<'a>(self) -> &'a [MTKey] {
        let count = self.key_count();
        &self.key_slots()[..count]
    }

    #[inline]
    pub fn key(self, i: usize) -> MTKey {
        self.key_slots()[i]
    }

    #[inline]
    pub fn set_key(self, i: usize, key: MTKey) {
        self.key_slots()[i] = key;
    }

    /// Edit a key in place — the shape the position rebasing wants, which reads
    /// and writes `key.pos` through a `&mut MTPos`.
    #[inline]
    pub fn update_key<T>(self, i: usize, edit: impl FnOnce(&mut MTKey) -> T) -> T {
        edit(&mut self.key_slots()[i])
    }

    /// Shift `src` of the key array to start at `dest`, as the C's `memmove`
    /// did. Overlapping ranges are fine; the key count is the caller's to fix.
    #[inline]
    pub fn copy_keys_within(self, src: Range<usize>, dest: usize) {
        self.key_slots().copy_within(src, dest);
    }

    /// Copy `src`'s keys in `range` into this node starting at `dest`.
    #[inline]
    pub fn copy_keys_from(self, dest: usize, src: Node, range: Range<usize>) {
        debug_assert!(self != src, "self != src");
        let count = range.len();
        let from = src.key_slots();
        self.key_slots()[dest..dest + count].copy_from_slice(&from[range]);
    }

    // -- children ----------------------------------------------------------

    /// The tail an internal node carries past its header.
    ///
    /// # Panics (debug)
    /// A leaf has no tail: the allocation stops at `size_of::<MTNode>()`.
    #[inline]
    fn tail<'a>(self) -> &'a mut mtnode_inner_s {
        debug_assert!(!self.is_leaf(), "!self.is_leaf()");
        // SAFETY: `self` is live and internal, so it was allocated with `ILEN`
        // bytes and the tail is initialised storage; `inner` derives it from
        // the same raw pointer rather than through any reference.
        unsafe { &mut *inner(self.0) }
    }

    /// The `i`th subtree. Valid for `i <= key_count()`.
    #[inline]
    pub fn child(self, i: usize) -> Node {
        debug_assert!(i <= self.key_count(), "i <= key_count");
        Node(self.tail().i_ptr[i])
    }

    #[inline]
    pub fn set_child(self, i: usize, child: Node) {
        self.tail().i_ptr[i] = child.as_ptr();
    }

    /// How many keys of each meta kind the `i`th subtree holds.
    #[inline]
    pub fn child_meta(self, i: usize) -> MetaCount {
        self.tail().i_meta[i]
    }

    #[inline]
    pub fn set_child_meta(self, i: usize, meta: MetaCount) {
        self.tail().i_meta[i] = meta;
    }

    /// Edit one child's counts in place — the shape `meta_add`/`meta_sub` want.
    #[inline]
    pub fn update_child_meta<T>(self, i: usize, edit: impl FnOnce(&mut MetaCount) -> T) -> T {
        edit(&mut self.tail().i_meta[i])
    }

    /// Shift `src` of the child tail — pointers and counts together — to start
    /// at `dest`. The children's own `p_idx` is the caller's to fix, through
    /// [`Node::reparent_children`].
    #[inline]
    pub fn copy_children_within(self, src: Range<usize>, dest: usize) {
        self.tail().i_ptr.copy_within(src.clone(), dest);
        self.tail().i_meta.copy_within(src, dest);
    }

    /// Copy `src`'s children in `range` — pointers and counts — into this node
    /// starting at `dest`.
    #[inline]
    pub fn copy_children_from(self, dest: usize, src: Node, range: Range<usize>) {
        debug_assert!(self != src, "self != src");
        let count = range.len();
        let tail = src.tail();
        let to = dest..dest + count;
        self.tail().i_ptr[to.clone()].copy_from_slice(&tail.i_ptr[range.clone()]);
        self.tail().i_meta[to].copy_from_slice(&tail.i_meta[range]);
    }

    /// Tell every child in `range` where it now sits, after the tail moved.
    #[inline]
    pub fn reparent_children(self, range: Range<usize>) {
        for i in range {
            let child = self.child(i);
            child.set_parent(Some(self));
            child.set_parent_index(i);
        }
    }

    // -- the intersection set ----------------------------------------------

    /// The ids of the paired marks whose ranges cover the whole of this node.
    #[inline]
    pub fn intersection(self) -> IdSet {
        // SAFETY: `self` is live, so its `intersect` outlives the view; the
        // view is derived from the node pointer and not from any reference
        // covering the set's own inline array.
        unsafe { IdSet::new(&raw mut (*self.0).intersect) }
    }

    // -- derived -----------------------------------------------------------

    /// Recompute the node's own meta counts from its keys and its children's.
    pub fn meta(self) -> MetaCount {
        let mut meta = [0; META_COUNT];
        for i in 0..self.key_count() {
            meta_add_key(&mut meta, self.key(i));
        }
        if !self.is_leaf() {
            for i in 0..=self.key_count() {
                meta_add(&mut meta, &self.child_meta(i));
            }
        }
        meta
    }

    /// An ordering key for "the gap just before index `i` of this node": the
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
    pub fn pseudo_index(self, i: c_int) -> uint64_t {
        let mut index: uint64_t = 0;
        let mut off = MT_LOG2_BRANCH * self.level() as uint32_t;
        let (mut node, mut i) = (Some(self), i);
        while let Some(x) = node {
            index |= ((i + 1) as uint64_t) << off;
            off += MT_LOG2_BRANCH;
            i = x.parent_index() as c_int;
            node = x.parent();
        }
        index
    }
}

/// The child pointers and per-child meta counts of an internal node.
///
/// Private: [`Node::child`] and its neighbours are how the rest of the family
/// reaches the tail, and they keep the index within the node's own count.
///
/// # Safety
/// `x` must be a live internal node — one allocated with [`ILEN`] bytes, i.e.
/// one whose `level` is non-zero.
#[inline]
unsafe fn inner(x: *mut MTNode) -> *mut mtnode_inner_s {
    // SAFETY: the caller promises `x` is a live internal node, so the tail is
    // within the allocation and `s` names its first byte.
    unsafe { (&raw mut (*x).s).cast() }
}

/// Allocate a zeroed node with its intersection set pointed at its own inline
/// array, and count it against the tree.
///
/// # Safety
/// `b` must be a live tree.
pub unsafe fn marktree_alloc_node(b: *mut MarkTree, internal: bool) -> *mut MTNode {
    let bytes = if internal { ILEN } else { size_of::<MTNode>() };
    // SAFETY: `xcalloc` hands back `bytes` of zeroed, suitably aligned storage
    // (it never returns null), which is exactly a fresh node of that shape.
    let node = unsafe { Node::new(xcalloc(1, bytes).cast()) };
    node.intersection().init();
    // SAFETY: `b` is live per the caller, and nothing else borrows it.
    unsafe { (*b).n_nodes += 1 };
    node.as_ptr()
}

/// Free one node, and its intersection set's heap buffer if it ever grew one.
///
/// # Safety
/// `b` must be a live tree and `x` one of its live nodes, detached from it —
/// nothing may name `x` afterwards.
pub unsafe fn marktree_free_node(b: *mut MarkTree, x: *mut MTNode) {
    // SAFETY: the caller promises `x` is a live node of `b`.
    let heap = unsafe { Node::new(x) }.intersection().take_heap();
    // SAFETY: `take_heap` hands back the set's own heap buffer, or null if it
    // never left the inline array; either is `xfree`-able exactly once.
    unsafe { xfree(heap) };
    // SAFETY: `x` came from `marktree_alloc_node`'s `xcalloc` and is detached.
    unsafe { xfree(x.cast()) };
    // SAFETY: `b` is live per the caller, and nothing else borrows it.
    unsafe { (*b).n_nodes -= 1 };
}

/// Record that `x` is the node holding key `i`, so `marktree_lookup` can find
/// it by id without a walk.
///
/// # Safety
/// `b` must be a live tree and `x` one of its live nodes, holding a key at `i`.
#[inline]
pub unsafe fn refkey(b: *mut MarkTree, x: *mut MTNode, i: c_int) {
    // SAFETY: the caller promises `x` is a live node holding a key at `i`.
    let node = unsafe { Node::new(x) };
    let id = mt_lookup_key(node.key(i as usize));
    // SAFETY: `b` is live, so its one-element `id2node` array is a live map.
    let map: *mut Map_uint64_t_ptr_t = unsafe { &raw mut (*b).id2node }.cast();
    let (init, fresh) = (ptr::null_mut(), ptr::null_mut());
    // SAFETY: `map` is a live map; the two nulls decline its optional
    // "initial value" and "was it new" out-parameters.
    let slot = unsafe { map_put_ref_uint64_t_ptr_t(map, id, init, fresh) };
    // SAFETY: `map_put_ref` answers a live slot of the map it was handed.
    unsafe { *slot = node.as_ptr() as ptr_t };
}

/// The node holding the key with this lookup handle, or null.
///
/// # Safety
/// `b` must be a live tree.
pub unsafe fn id2node(b: *mut MarkTree, id: uint64_t) -> *mut MTNode {
    // SAFETY: `b` is live, so its one-element `id2node` array is a live map.
    let map: *mut Map_uint64_t_ptr_t = unsafe { &raw mut (*b).id2node }.cast();
    // SAFETY: `map` is live, so its `set` is the map's own live key set.
    let k = unsafe { mh_get_uint64_t(&raw mut (*map).set, id) };
    if k == uint32_t::MAX {
        return ptr::null_mut();
    }
    // SAFETY: a hash index the set answered is in bounds of `map.values`, and
    // every value in this map was stored by `refkey` as a node pointer.
    unsafe { (*(*map).values.add(k as usize)).cast() }
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
        return (keys.len() as c_int - 1, false);
    }
    let found = key_cmp(k, keys[begin]) == 0;
    (begin as c_int - c_int::from(!found), found)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::marktree::key::{DECOR_HIGHLIGHT_INLINE_INIT, MT_FLAG_REAL, MT_FLAG_RIGHT_GRAVITY};
    use crate::types::{DecorInlineData, MTPos, uint16_t};

    fn key(row: i32, col: i32, flags: c_int) -> MTKey {
        MTKey {
            pos: MTPos { row, col },
            ns: 0,
            id: 0,
            flags: flags as uint16_t,
            decor_data: DecorInlineData {
                hl: DECOR_HIGHLIGHT_INLINE_INIT,
            },
        }
    }

    #[test]
    fn an_internal_node_is_one_child_tail_larger_than_a_leaf() {
        assert_eq!(ILEN - size_of::<MTNode>(), size_of::<mtnode_inner_s>());
        // The tail starts where the struct ends, which is what makes a
        // reference to the struct too short to reach it.
        assert_eq!(size_of::<MTNode>() % align_of::<mtnode_inner_s>(), 0);
    }

    #[test]
    fn the_tail_holds_exactly_max_children_of_each() {
        // `MAX_KEYS` needs no test: `Node::key_slots` names the key array's
        // type, so a mismatch is a compile error. The tail is only indexed, so
        // its capacity is checked by size instead.
        let per_child = size_of::<*mut MTNode>() + size_of::<MetaCount>();
        assert_eq!(size_of::<mtnode_inner_s>(), MAX_CHILDREN * per_child);
    }

    #[test]
    fn an_empty_node_answers_before_everything() {
        assert_eq!(find_key(&[], key(0, 0, MT_FLAG_REAL)), (-1, false));
    }

    #[test]
    fn finds_an_exact_match_and_reports_it() {
        let keys = [
            key(1, 0, MT_FLAG_REAL),
            key(1, 5, MT_FLAG_REAL),
            key(3, 2, MT_FLAG_REAL),
        ];
        assert_eq!(find_key(&keys, key(1, 5, MT_FLAG_REAL)), (1, true));
        assert_eq!(find_key(&keys, key(3, 2, MT_FLAG_REAL)), (2, true));
    }

    #[test]
    fn a_key_between_two_others_lands_on_the_earlier_one() {
        let keys = [
            key(1, 0, MT_FLAG_REAL),
            key(1, 5, MT_FLAG_REAL),
            key(3, 2, MT_FLAG_REAL),
        ];
        assert_eq!(find_key(&keys, key(1, 3, MT_FLAG_REAL)), (0, false));
        assert_eq!(find_key(&keys, key(9, 9, MT_FLAG_REAL)), (2, false));
        // Before the first key at all.
        assert_eq!(find_key(&keys, key(0, 0, MT_FLAG_REAL)), (-1, false));
    }

    #[test]
    fn gravity_breaks_a_tie_the_way_the_order_does() {
        let keys = [
            key(1, 1, MT_FLAG_REAL),
            key(1, 1, MT_FLAG_REAL | MT_FLAG_RIGHT_GRAVITY),
        ];
        assert_eq!(
            find_key(&keys, key(1, 1, MT_FLAG_REAL | MT_FLAG_RIGHT_GRAVITY)),
            (1, true)
        );
        assert_eq!(find_key(&keys, key(1, 1, MT_FLAG_REAL)), (0, true));
    }
}
