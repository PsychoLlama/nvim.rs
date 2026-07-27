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
//! `test/unit/marktree_spec.lua` pins the layout besides: it reads
//! `tree[0].root.level` and `iter[0].x.key[iter[0].i]` through LuaJIT's FFI.

use crate::src::nvim::map::{map_put_ref_uint64_t_ptr_t, mh_get_uint64_t};
use crate::src::nvim::memory::{xcalloc, xfree};
use crate::src::nvim::types::{
    MTKey, MTNode, MarkTree, mtnode_inner_s, ptr_t, size_t, uint32_t, uint64_t,
};

use super::key::{MT_LOG2_BRANCH, key_cmp, mt_lookup_key};
use super::meta::{MetaCount, meta_add_key};

/// Size of an internal node: the common header plus the child tail.
pub const ILEN: usize = size_of::<MTNode>() + size_of::<mtnode_inner_s>();

/// How many ids a node's intersection set holds before it goes to the heap.
/// Nested to keep the name out of the flat cdef namespace `ffigen` builds.
mod sizes {
    pub const INTERSECT_INLINE: usize = 4;
}
pub use sizes::INTERSECT_INLINE;

/// The child pointers and per-child meta counts of an internal node.
///
/// # Safety
/// `x` must be a live internal node — one allocated with [`ILEN`] bytes, i.e.
/// one whose `level` is non-zero.
#[inline]
pub unsafe fn inner(x: *mut MTNode) -> *mut mtnode_inner_s {
    (&raw mut (*x).s).cast()
}

/// Allocate a zeroed node with its intersection set pointed at its own inline
/// array, and count it against the tree.
pub unsafe fn marktree_alloc_node(b: *mut MarkTree, internal: bool) -> *mut MTNode {
    let bytes = if internal { ILEN } else { size_of::<MTNode>() };
    let x: *mut MTNode = xcalloc(1, bytes).cast();
    (*x).intersect.capacity = INTERSECT_INLINE as size_t;
    (*x).intersect.size = 0;
    (*x).intersect.items = (&raw mut (*x).intersect.init_array).cast();
    (*b).n_nodes += 1;
    x
}

/// Free one node, and its intersection set's heap buffer if it ever grew one.
pub unsafe fn marktree_free_node(b: *mut MarkTree, x: *mut MTNode) {
    if (*x).intersect.items != (&raw mut (*x).intersect.init_array).cast() {
        xfree((*x).intersect.items.cast());
    }
    xfree(x.cast());
    (*b).n_nodes -= 1;
}

/// Record that `x` is the node holding key `i`, so `marktree_lookup` can find
/// it by id without a walk.
#[inline]
pub unsafe fn refkey(b: *mut MarkTree, x: *mut MTNode, i: ::core::ffi::c_int) {
    let id = mt_lookup_key((*x).key[i as usize]);
    let slot = map_put_ref_uint64_t_ptr_t(
        &raw mut (*b).id2node as *mut _,
        id,
        ::core::ptr::null_mut(),
        ::core::ptr::null_mut(),
    );
    *slot = x as ptr_t;
}

/// The node holding the key with this lookup handle, or null.
pub unsafe fn id2node(b: *mut MarkTree, id: uint64_t) -> *mut MTNode {
    let map = &raw mut (*b).id2node as *mut crate::src::nvim::types::Map_uint64_t_ptr_t;
    let k = mh_get_uint64_t(&raw mut (*map).set, id);
    if k == uint32_t::MAX {
        ::core::ptr::null_mut()
    } else {
        (*(*map).values.offset(k as isize)).cast()
    }
}

/// Recompute a node's own meta counts from its keys and its children's counts.
pub unsafe fn meta_describe_node(x: *mut MTNode) -> MetaCount {
    let mut meta = [0; super::meta::META_COUNT];
    for i in 0..(*x).n as usize {
        meta_add_key(&mut meta, (*x).key[i]);
    }
    if (*x).level != 0 {
        for i in 0..=(*x).n as usize {
            super::meta::meta_add(&mut meta, &(*inner(x)).i_meta[i]);
        }
    }
    meta
}

/// An ordering key for "the gap just before index `i` of node `x`": the path
/// from the root packed [`MT_LOG2_BRANCH`] bits per level, root in the high
/// bits.
///
/// The first index is shifted by the *node's own level* rather than by zero, so
/// that indices taken at different depths are still comparable — a leaf's path
/// occupies the low bits and an internal node's stops short of them, which is
/// what makes "before this whole subtree" compare less than anything inside it.
/// Two of these compare exactly as the positions they name do, which is what
/// lets the overlap walk decide whether an intersecting pair started before the
/// iterator without walking to it. Depth is bounded by `MT_MAX_DEPTH` and the
/// branch by `2 * MT_BRANCH_FACTOR`, so the path fits in 64 bits with room for
/// the "immediately before this node" index zero.
pub unsafe fn pseudo_index(mut x: *mut MTNode, mut i: ::core::ffi::c_int) -> uint64_t {
    let mut index: uint64_t = 0;
    let mut off = MT_LOG2_BRANCH as ::core::ffi::c_int * (*x).level as ::core::ffi::c_int;
    while !x.is_null() {
        index |= ((i + 1) as uint64_t) << off;
        off += MT_LOG2_BRANCH as ::core::ffi::c_int;
        i = (*x).p_idx as ::core::ffi::c_int;
        x = (*x).parent;
    }
    index
}

/// Where key `k` belongs among `keys`, which are sorted.
///
/// Answers the index of the last key that is not greater than `k`, and whether
/// that key compares equal. An empty node answers `-1`, and so does a `k` that
/// sorts before every key present — the caller reads the result as "the gap
/// after index `n`", so `-1` means "before everything".
pub fn find_key(keys: &[MTKey], k: MTKey) -> (::core::ffi::c_int, bool) {
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
        return (keys.len() as ::core::ffi::c_int - 1, false);
    }
    let found = key_cmp(k, keys[begin]) == 0;
    (
        begin as ::core::ffi::c_int - ::core::ffi::c_int::from(!found),
        found,
    )
}

/// The keys a node currently holds.
///
/// # Safety
/// `x` must be a live node.
#[inline]
pub unsafe fn node_keys<'a>(x: *const MTNode) -> &'a [MTKey] {
    ::core::slice::from_raw_parts(&raw const (*x).key as *const MTKey, (*x).n as usize)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::src::nvim::marktree::key::{
        DECOR_HIGHLIGHT_INLINE_INIT, MT_FLAG_REAL, MT_FLAG_RIGHT_GRAVITY,
    };
    use crate::src::nvim::types::{DecorInlineData, MTPos, uint16_t};

    fn key(row: i32, col: i32, flags: ::core::ffi::c_int) -> MTKey {
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
