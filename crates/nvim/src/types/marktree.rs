#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;
use crate::marktree::node::{NodeId, NodeSlab};
use crate::os::libc::{int32_t, size_t, uint32_t, uint64_t};
use crate::registry::{IdMap, id_map};

#[derive(Copy, Clone)]
#[repr(C)]
pub struct MTKey {
    pub pos: MTPos,
    pub ns: uint32_t,
    pub id: uint32_t,
    pub flags: crate::marktree::key::MtFlags,
    pub decor_data: DecorInline,
}
#[derive(Copy, Clone)]
pub struct MTPair {
    pub start: MTKey,
    pub end_pos: MTPos,
    pub end_right_gravity: bool,
}
#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct MTPos {
    pub row: int32_t,
    pub col: int32_t,
}
/// The extmark B-tree of one buffer.
///
/// Its nodes live in [`NodeSlab`], addressed by [`NodeId`]: a node names its
/// parent and children by id, and so does an iterator, so nothing outside the
/// slab holds a node's address.
pub struct MarkTree {
    /// The top node, or `None` for a tree that has never held a key or has
    /// been emptied by `marktree_clear`.
    pub(crate) root: Option<NodeId>,
    pub meta_root: [uint32_t; 5],
    pub n_keys: size_t,
    pub(crate) nodes: NodeSlab,
    /// Every key's lookup handle to the node it lives in, so a mark can be
    /// found without a walk. Asked, never iterated.
    pub(crate) id2node: IdMap<uint64_t, NodeId>,
}

impl MarkTree {
    /// A tree with nothing in it.
    pub const EMPTY: MarkTree = MarkTree {
        root: None,
        meta_root: [0; 5],
        n_keys: 0,
        nodes: NodeSlab::new(),
        id2node: id_map(),
    };
}

impl Default for MarkTree {
    fn default() -> Self {
        MarkTree::EMPTY
    }
}
/// A cursor into a [`MarkTree`].
///
/// `Copy`, and deliberately: forking an iterator to look ahead is what the
/// marktree code does with it. It names its node by [`NodeId`] and the path
/// down to it by child index, so it holds nothing a tree change can leave
/// dangling: a stale iterator reads a wrong key, never freed memory.
#[derive(Copy, Clone, Default)]
pub struct MarkTreeIter {
    /// The absolute position the keys of `node` are relative to.
    pub pos: MTPos,
    /// How far below the root `node` is.
    pub depth: ::core::ffi::c_int,
    /// The node the iterator stands in, or `None` once a walk has run off the
    /// end (or before one has started).
    pub node: Option<NodeId>,
    /// The key within `node`.
    pub index: ::core::ffi::c_int,
    /// The child index taken at each level above `node`.
    pub path: [PathStep; 20],
    pub intersect_idx: size_t,
    pub intersect_pos: MTPos,
    pub intersect_pos_x: MTPos,
}

impl MarkTreeIter {
    /// An iterator positioned nowhere — what a walk starts from.
    pub const EMPTY: MarkTreeIter = MarkTreeIter {
        pos: MTPos { row: 0, col: 0 },
        depth: 0,
        node: None,
        index: 0,
        path: [PathStep {
            old_col: 0,
            index: 0,
        }; 20],
        intersect_idx: 0,
        intersect_pos: MTPos { row: 0, col: 0 },
        intersect_pos_x: MTPos { row: 0, col: 0 },
    };

    /// Whether the iterator names a key at all.
    pub fn is_positioned(&self) -> bool {
        self.node.is_some()
    }
}

/// One level of a [`MarkTreeIter`]'s path: which child it descended into, and
/// the column its position had before it did.
#[derive(Copy, Clone, Default)]
pub struct PathStep {
    pub old_col: ::core::ffi::c_int,
    pub index: ::core::ffi::c_int,
}
pub type MetaIndex = ::core::ffi::c_uint;
