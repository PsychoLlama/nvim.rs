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
use crate::registry::{IdMap, id_map};

#[repr(C)]
pub struct Intersection {
    pub size: size_t,
    pub capacity: size_t,
    pub items: *mut uint64_t,
    pub init_array: [uint64_t; 4],
}
#[derive(Copy, Clone)]
#[repr(C)]
pub struct MTKey {
    pub pos: MTPos,
    pub ns: uint32_t,
    pub id: uint32_t,
    pub flags: crate::marktree::key::MtFlags,
    pub decor_data: DecorInlineData,
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
pub struct MarkTree {
    pub root: *mut MTNode,
    pub meta_root: [uint32_t; 5],
    pub n_keys: size_t,
    pub n_nodes: size_t,
    /// Every key's lookup handle to the node it lives in, so a mark can be
    /// found without a walk. Asked, never iterated.
    pub(crate) id2node: IdMap<uint64_t, *mut MTNode>,
}

impl MarkTree {
    /// A tree with nothing in it. Upstream started one from all-zero bytes;
    /// the table is the one field that is not a value of its type there.
    pub const EMPTY: MarkTree = MarkTree {
        root: ::core::ptr::null_mut(),
        meta_root: [0; 5],
        n_keys: 0,
        n_nodes: 0,
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
/// `Copy`, and deliberately: `x` is a borrow of a node the tree owns, and
/// forking an iterator to look ahead is what the marktree code does with it.
#[derive(Copy, Clone, Default)]
pub struct MarkTreeIter {
    pub pos: MTPos,
    pub lvl: ::core::ffi::c_int,
    pub x: *mut MTNode,
    pub i: ::core::ffi::c_int,
    pub s: [MarkTreeIter_s; 20],
    pub intersect_idx: size_t,
    pub intersect_pos: MTPos,
    pub intersect_pos_x: MTPos,
}
#[derive(Copy, Clone, Default)]
pub struct MarkTreeIter_s {
    pub oldcol: ::core::ffi::c_int,
    pub i: ::core::ffi::c_int,
}
pub type MetaFilter = *const uint32_t;
pub type MetaIndex = ::core::ffi::c_uint;
#[repr(C)]
pub struct mtnode_inner_s {
    pub i_ptr: [*mut MTNode; 20],
    pub i_meta: [[uint32_t; 5]; 20],
}
#[repr(C)]
pub struct mtnode_s {
    pub n: int32_t,
    pub level: int16_t,
    pub p_idx: int16_t,
    pub intersect: Intersection,
    pub parent: *mut MTNode,
    pub key: [MTKey; 19],
    pub s: [mtnode_inner_s; 0],
}
