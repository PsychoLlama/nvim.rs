//! B-tree data structure for marks at (row, col) positions
//!
//! This crate provides Rust implementations of the marktree subsystem
//! from `src/nvim/marktree.c`. The marktree is a B-tree for storing
//! extmarks at positions and efficiently updating them for text changes.
//!
//! Uses an opaque handle pattern where C pointers are treated as opaque
//! handles, with field access done through C accessor functions.

#![allow(unsafe_code)] // FFI requires unsafe
#![allow(clippy::doc_markdown)] // Allow type names without backticks
#![allow(clippy::wildcard_imports)] // We use wildcard for flag constants
#![allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
#![allow(clippy::not_unsafe_ptr_arg_deref)] // FFI functions take raw pointers
#![allow(clippy::fn_params_excessive_bools)] // Matching C API signatures
#![allow(clippy::items_after_statements)] // Allow const definitions in functions

pub mod delete;
pub mod ffi;
pub mod insert;
pub mod intersect_ops;
pub mod intersection;
pub mod iter;
pub mod node;
pub mod overlap;
pub mod splice;
pub mod validate;

use std::ffi::c_int;

// Re-export native types for external use
pub use node::{MTNode, MarkTree, MarkTreeIter, MetaIndex};

// ============================================================================
// Constants
// ============================================================================

/// Maximum tree depth.
pub const MT_MAX_DEPTH: usize = 20;

/// Branch factor for the B-tree.
pub const MT_BRANCH_FACTOR: usize = 10;

/// Log2 of branch factor (for pseudo-index calculations).
pub const MT_LOG2_BRANCH: usize = 5;

/// End flag for mark lookup IDs.
pub const MARKTREE_END_FLAG: u64 = 1;

// ============================================================================
// Flag Constants
// ============================================================================

/// Flags for MTKey entries.
pub mod flags {
    /// Mark is a real mark (not a pseudo-key).
    pub const MT_FLAG_REAL: u16 = 1 << 0;
    /// Mark is the end of a paired range.
    pub const MT_FLAG_END: u16 = 1 << 1;
    /// Mark is part of a start/end pair.
    pub const MT_FLAG_PAIRED: u16 = 1 << 2;
    /// Other side of paired mark was deleted.
    pub const MT_FLAG_ORPHANED: u16 = 1 << 3;
    /// Mark should not be undone.
    pub const MT_FLAG_NO_UNDO: u16 = 1 << 4;
    /// Mark can be invalidated.
    pub const MT_FLAG_INVALIDATE: u16 = 1 << 5;
    /// Mark is currently invalid.
    pub const MT_FLAG_INVALID: u16 = 1 << 6;
    /// Decoration data is external (pointer).
    pub const MT_FLAG_DECOR_EXT: u16 = 1 << 7;
    /// Mark has highlight decoration.
    pub const MT_FLAG_DECOR_HL: u16 = 1 << 8;
    /// Mark has sign text decoration.
    pub const MT_FLAG_DECOR_SIGNTEXT: u16 = 1 << 9;
    /// Mark has sign highlight decoration.
    pub const MT_FLAG_DECOR_SIGNHL: u16 = 1 << 10;
    /// Mark has virtual lines decoration.
    pub const MT_FLAG_DECOR_VIRT_LINES: u16 = 1 << 11;
    /// Mark has inline virtual text decoration.
    pub const MT_FLAG_DECOR_VIRT_TEXT_INLINE: u16 = 1 << 12;
    /// Mark has concealed lines decoration.
    pub const MT_FLAG_DECOR_CONCEAL_LINES: u16 = 1 << 13;
    /// Mark has right gravity (moves with insertions at same position).
    pub const MT_FLAG_RIGHT_GRAVITY: u16 = 1 << 14;
    /// Last flag (for ordering).
    pub const MT_FLAG_LAST: u16 = 1 << 15;

    /// Mask for decoration-related flags.
    pub const MT_FLAG_DECOR_MASK: u16 = MT_FLAG_DECOR_EXT
        | MT_FLAG_DECOR_HL
        | MT_FLAG_DECOR_SIGNTEXT
        | MT_FLAG_DECOR_SIGNHL
        | MT_FLAG_DECOR_VIRT_LINES
        | MT_FLAG_DECOR_VIRT_TEXT_INLINE;

    /// Mask for externally modifiable flags.
    pub const MT_FLAG_EXTERNAL_MASK: u16 = MT_FLAG_DECOR_MASK
        | MT_FLAG_NO_UNDO
        | MT_FLAG_INVALIDATE
        | MT_FLAG_INVALID
        | MT_FLAG_DECOR_CONCEAL_LINES;
}

use flags::*;

// ============================================================================
// Types
// ============================================================================

/// Position in the buffer (row, col).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MTPos {
    pub row: i32,
    pub col: i32,
}

impl MTPos {
    /// Create a new position.
    #[inline]
    #[must_use]
    pub const fn new(row: i32, col: i32) -> Self {
        Self { row, col }
    }
}

/// Opaque representation of C's `DecorInlineData` union (16 bytes).
///
/// In C this is a union of `DecorHighlightInline` (12 bytes) and
/// `DecorExt` (16 bytes: `uint32_t sh_idx` + `DecorVirtText *vt`).
/// We store the raw bytes as two `u64` values for FFI compatibility.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct DecorInlineData {
    pub data: [u64; 2],
}

impl DecorInlineData {
    /// Create a zero/empty decoration data.
    #[inline]
    #[must_use]
    pub const fn zero() -> Self {
        Self { data: [0, 0] }
    }
}

/// Key for a mark in the tree.
///
/// The `decor_data` field matches C's `DecorInlineData` union (16 bytes).
/// The actual interpretation depends on the MT_FLAG_DECOR_EXT flag.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MTKey {
    pub pos: MTPos,
    pub ns: u32,
    pub id: u32,
    pub flags: u16,
    /// Decoration data (union in C: DecorHighlightInline or DecorExt).
    pub decor_data: DecorInlineData,
}

impl Default for MTKey {
    fn default() -> Self {
        Self {
            pos: MTPos::new(-1, -1),
            ns: 0,
            id: 0,
            flags: 0,
            decor_data: DecorInlineData::zero(),
        }
    }
}

impl MTKey {
    /// Create a zero-initialized key.
    #[inline]
    #[must_use]
    pub const fn zero() -> Self {
        Self {
            pos: MTPos::new(0, 0),
            ns: 0,
            id: 0,
            flags: 0,
            decor_data: DecorInlineData::zero(),
        }
    }

    /// Create an invalid key sentinel.
    #[inline]
    #[must_use]
    pub const fn invalid() -> Self {
        Self {
            pos: MTPos { row: -1, col: -1 },
            ns: 0,
            id: 0,
            flags: 0,
            decor_data: DecorInlineData::zero(),
        }
    }

    /// Check if this key is valid.
    #[inline]
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.pos.row >= 0 && self.pos.col >= 0
    }
}

// ============================================================================
// Opaque Handle Types
// ============================================================================

/// Opaque handle to a MarkTree (`MarkTree*`).
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MarkTreeHandle(*mut std::ffi::c_void);

impl MarkTreeHandle {
    /// Check if the handle is null.
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to an MTNode (`MTNode*`).
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MTNodeHandle(*mut std::ffi::c_void);

impl MTNodeHandle {
    /// Check if the handle is null.
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Create a null handle.
    #[inline]
    #[must_use]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }
}

/// Opaque handle to a MarkTreeIter (`MarkTreeIter*`).
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MarkTreeIterHandle(*mut std::ffi::c_void);

impl MarkTreeIterHandle {
    /// Check if the handle is null.
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }

    /// Create a null handle.
    #[inline]
    #[must_use]
    pub const fn null() -> Self {
        Self(std::ptr::null_mut())
    }

    /// Create a handle from a raw pointer.
    ///
    /// # Safety
    /// The pointer must be a valid `MarkTreeIter*` or null.
    #[inline]
    #[must_use]
    pub const unsafe fn from_ptr(ptr: *mut std::ffi::c_void) -> Self {
        Self(ptr)
    }

    /// Get the raw pointer.
    #[inline]
    #[must_use]
    pub const fn as_ptr(self) -> *mut std::ffi::c_void {
        self.0
    }
}

// ============================================================================
// C Accessor Functions
// ============================================================================

extern "C" {
    // ========================================================================
    // Node accessors
    // ========================================================================

    /// Get the number of keys in a node.
    fn nvim_mtnode_get_n(x: MTNodeHandle) -> c_int;

    /// Get the level of a node (0 for leaf).
    fn nvim_mtnode_get_level(x: MTNodeHandle) -> c_int;

    /// Get a key from a node by index.
    fn nvim_mtnode_get_key(x: MTNodeHandle, idx: c_int) -> MTKey;

    /// Get a child pointer from a node by index.
    fn nvim_mtnode_get_ptr(x: MTNodeHandle, idx: c_int) -> MTNodeHandle;

    /// Get the parent of a node.
    fn nvim_mtnode_get_parent(x: MTNodeHandle) -> MTNodeHandle;

    /// Get the parent index of a node.
    #[allow(dead_code)]
    fn nvim_mtnode_get_p_idx(x: MTNodeHandle) -> c_int;

    // ========================================================================
    // Tree accessors
    // ========================================================================

    /// Get the root node of a marktree.
    fn nvim_marktree_get_root(b: MarkTreeHandle) -> MTNodeHandle;

    /// Get the total number of keys in a marktree.
    fn nvim_marktree_get_n_keys(b: MarkTreeHandle) -> usize;

    /// Get the root level of a marktree.
    #[allow(dead_code)]
    fn nvim_marktree_get_root_level(b: MarkTreeHandle) -> c_int;

    // ========================================================================
    // Iterator accessors
    // ========================================================================

    /// Get the current node from an iterator.
    fn nvim_mtitr_get_x(itr: MarkTreeIterHandle) -> MTNodeHandle;

    /// Get the current index from an iterator.
    fn nvim_mtitr_get_i(itr: MarkTreeIterHandle) -> c_int;

    /// Get the current level from an iterator.
    fn nvim_mtitr_get_lvl(itr: MarkTreeIterHandle) -> c_int;

    /// Get the current position from an iterator.
    fn nvim_mtitr_get_pos(itr: MarkTreeIterHandle) -> MTPos;

    /// Set the current node in an iterator.
    fn nvim_mtitr_set_x(itr: MarkTreeIterHandle, x: MTNodeHandle);

    /// Set the current index in an iterator.
    fn nvim_mtitr_set_i(itr: MarkTreeIterHandle, i: c_int);

    /// Set the current level in an iterator.
    fn nvim_mtitr_set_lvl(itr: MarkTreeIterHandle, lvl: c_int);

    /// Set the current position in an iterator.
    fn nvim_mtitr_set_pos(itr: MarkTreeIterHandle, pos: MTPos);

    /// Get stored index at level from an iterator.
    fn nvim_mtitr_get_s_i(itr: MarkTreeIterHandle, lvl: c_int) -> c_int;

    /// Get stored oldcol at level from an iterator.
    fn nvim_mtitr_get_s_oldcol(itr: MarkTreeIterHandle, lvl: c_int) -> c_int;

    /// Set stored index at level in an iterator.
    fn nvim_mtitr_set_s_i(itr: MarkTreeIterHandle, lvl: c_int, i: c_int);

    /// Set stored oldcol at level in an iterator.
    fn nvim_mtitr_set_s_oldcol(itr: MarkTreeIterHandle, lvl: c_int, oldcol: c_int);

    // ========================================================================
    // Lookup and pair functions
    // ========================================================================

    /// Lookup a mark by its ID and optionally set iterator.
    #[allow(dead_code)]
    fn nvim_marktree_lookup(b: MarkTreeHandle, id: u64, itr: MarkTreeIterHandle) -> MTKey;

    /// Lookup a mark by namespace and ID.
    #[allow(dead_code)]
    fn nvim_marktree_lookup_ns(
        b: MarkTreeHandle,
        ns: u32,
        id: u32,
        end: bool,
        itr: MarkTreeIterHandle,
    ) -> MTKey;

    /// Get the alternate end of a paired mark.
    #[allow(dead_code)]
    fn nvim_marktree_get_alt(b: MarkTreeHandle, mark: MTKey, itr: MarkTreeIterHandle) -> MTKey;

    /// Get the position of the alternate end of a paired mark.
    #[allow(dead_code)]
    fn nvim_marktree_get_altpos(b: MarkTreeHandle, mark: MTKey, itr: MarkTreeIterHandle) -> MTPos;

    // ========================================================================
    // Iterator Overlap Accessors
    // ========================================================================

    /// Get intersect_pos from an iterator.
    fn nvim_mtitr_get_intersect_pos(itr: MarkTreeIterHandle) -> MTPos;

    /// Set intersect_pos in an iterator.
    fn nvim_mtitr_set_intersect_pos(itr: MarkTreeIterHandle, pos: MTPos);

    /// Get intersect_pos_x from an iterator.
    fn nvim_mtitr_get_intersect_pos_x(itr: MarkTreeIterHandle) -> MTPos;

    /// Set intersect_pos_x in an iterator.
    fn nvim_mtitr_set_intersect_pos_x(itr: MarkTreeIterHandle, pos: MTPos);

    /// Get intersect_idx from an iterator.
    fn nvim_mtitr_get_intersect_idx(itr: MarkTreeIterHandle) -> usize;

    /// Set intersect_idx in an iterator.
    fn nvim_mtitr_set_intersect_idx(itr: MarkTreeIterHandle, idx: usize);

    // ========================================================================
    // Node Intersection Accessors
    // ========================================================================

    /// Get the size of the intersect array in a node.
    fn nvim_mtnode_get_intersect_size(x: MTNodeHandle) -> usize;

    /// Get an element from the intersect array in a node.
    fn nvim_mtnode_get_intersect_elem(x: MTNodeHandle, idx: usize) -> u64;

    /// Get a meta count from a node (for internal nodes only).
    fn nvim_mtnode_get_meta(x: MTNodeHandle, idx: c_int, m: c_int) -> u32;

    /// Get the meta_root array from a marktree.
    fn nvim_marktree_get_meta_root(b: MarkTreeHandle, meta_out: *mut u32);

    /// Check if meta filter matches.
    fn nvim_meta_has(meta_count: *const u32, meta_filter: *const u32) -> bool;

    /// Get the id for lookup from a node's intersect.
    #[allow(dead_code)]
    fn nvim_mtnode_intersect_id(x: MTNodeHandle, idx: usize) -> u64;

    /// Lookup node by id.
    fn nvim_marktree_id2node(b: MarkTreeHandle, id: u64) -> MTNodeHandle;

    /// Return the number of entries in the id2node map.
    fn nvim_marktree_id2node_count(b: MarkTreeHandle) -> usize;

    // ========================================================================
    // Helper Functions
    // ========================================================================

    /// Set iterator to point at node n, index i.
    #[allow(dead_code)]
    fn nvim_marktree_itr_set_node(
        b: MarkTreeHandle,
        itr: MarkTreeIterHandle,
        n: MTNodeHandle,
        i: c_int,
    ) -> MTKey;

    /// Fix iterator position after setting node directly.
    #[allow(dead_code)]
    fn nvim_marktree_itr_fix_pos(b: MarkTreeHandle, itr: MarkTreeIterHandle);

    // ========================================================================
    // Node Mutation Functions (Phase 4)
    // ========================================================================

    /// Set the number of keys in a node.
    fn nvim_mtnode_set_n(x: MTNodeHandle, n: c_int);

    /// Set the level of a node.
    fn nvim_mtnode_set_level(x: MTNodeHandle, level: c_int);

    /// Set a key in a node at the given index.
    fn nvim_mtnode_set_key(x: MTNodeHandle, idx: c_int, k: MTKey);

    /// Set a child pointer in a node at the given index.
    fn nvim_mtnode_set_ptr(x: MTNodeHandle, idx: c_int, ptr: MTNodeHandle);

    /// Set the parent of a node.
    fn nvim_mtnode_set_parent(x: MTNodeHandle, parent: MTNodeHandle);

    /// Set the parent index of a node.
    fn nvim_mtnode_set_p_idx(x: MTNodeHandle, p_idx: c_int);

    /// Set a meta count for a node child.
    fn nvim_mtnode_set_meta(x: MTNodeHandle, idx: c_int, m: c_int, val: u32);

    /// Move keys within a node (memmove).
    fn nvim_mtnode_memmove_keys(x: MTNodeHandle, dst: c_int, src: c_int, count: c_int);

    /// Move child pointers within a node (memmove).
    fn nvim_mtnode_memmove_ptr(x: MTNodeHandle, dst: c_int, src: c_int, count: c_int);

    /// Move meta arrays within a node (memmove).
    fn nvim_mtnode_memmove_meta(x: MTNodeHandle, dst: c_int, src: c_int, count: c_int);

    /// Copy keys from one node to another.
    fn nvim_mtnode_memcpy_keys(
        dst: MTNodeHandle,
        dst_idx: c_int,
        src: MTNodeHandle,
        src_idx: c_int,
        count: c_int,
    );

    /// Copy child pointers from one node to another.
    fn nvim_mtnode_memcpy_ptr(
        dst: MTNodeHandle,
        dst_idx: c_int,
        src: MTNodeHandle,
        src_idx: c_int,
        count: c_int,
    );

    /// Copy meta arrays from one node to another.
    fn nvim_mtnode_memcpy_meta(
        dst: MTNodeHandle,
        dst_idx: c_int,
        src: MTNodeHandle,
        src_idx: c_int,
        count: c_int,
    );

    // ========================================================================
    // Tree Mutation Functions (Phase 4)
    // ========================================================================

    /// Allocate a new marktree node.
    fn nvim_marktree_alloc_node(b: MarkTreeHandle, internal: bool) -> MTNodeHandle;

    /// Update id2node map for a key at given index.
    fn nvim_marktree_refkey(b: MarkTreeHandle, x: MTNodeHandle, i: c_int);

    /// Set the root node of a marktree.
    fn nvim_marktree_set_root(b: MarkTreeHandle, root: MTNodeHandle);

    /// Increment the number of keys in a marktree.
    fn nvim_marktree_inc_n_keys(b: MarkTreeHandle);

    /// Add to meta_root by index.
    fn nvim_marktree_add_meta_root(b: MarkTreeHandle, m: c_int, val: u32);

    /// Set meta_root by index.
    #[allow(dead_code)]
    fn nvim_marktree_set_meta_root(b: MarkTreeHandle, m: c_int, val: u32);

    // ========================================================================
    // Intersection Operations (Phase 4)
    // ========================================================================

    /// Copy intersections from one node to another.
    fn nvim_kvi_copy_intersect(dst: MTNodeHandle, src: MTNodeHandle);

    /// Clear intersections in a node.
    #[allow(dead_code)]
    fn nvim_kvi_init_intersect(x: MTNodeHandle);

    // ========================================================================
    // Intersection Mutation Accessors (Phase 1)
    // ========================================================================

    /// Clear and reinitialize a node's intersection list (destroy + kvi_init).
    fn nvim_mtnode_intersect_clear(x: MTNodeHandle);

    /// Push an ID onto a node's intersection list (unsorted, for rebuilding).
    fn nvim_mtnode_intersect_push(x: MTNodeHandle, id: u64);

    // ========================================================================
    // ========================================================================
    // B-tree Deletion Operations (Phase 5)
    // ========================================================================

    /// Delete mark at iterator position.
    #[allow(dead_code)]
    fn nvim_marktree_del_itr(b: MarkTreeHandle, itr: MarkTreeIterHandle, rev: bool) -> u64;

    /// Delete key from id2node map.
    fn nvim_marktree_del_id(b: MarkTreeHandle, id: u64);

    /// Decrement the number of keys in a marktree.
    fn nvim_marktree_dec_n_keys(b: MarkTreeHandle);

    /// Subtract from meta_root by index.
    fn nvim_marktree_sub_meta_root(b: MarkTreeHandle, m: c_int, val: u32);

    /// Get the raw key at iterator position.
    #[allow(dead_code)]
    fn nvim_rawkey(itr: MarkTreeIterHandle) -> MTKey;

    /// Set flags on the raw key at iterator position.
    fn nvim_rawkey_set_flags(itr: MarkTreeIterHandle, flags: u16);

    /// OR flags on the raw key at iterator position.
    fn nvim_rawkey_or_flags(itr: MarkTreeIterHandle, flags: u16);

    /// AND-NOT flags on the raw key at iterator position.
    fn nvim_rawkey_clear_flags(itr: MarkTreeIterHandle, flags: u16);

    /// Set the pos field of the key at the iterator position.
    fn nvim_rawkey_set_pos(itr: MarkTreeIterHandle, pos: MTPos);

    /// Get the pos field of the key at the iterator position.
    #[allow(dead_code)]
    fn nvim_rawkey_get_pos(itr: MarkTreeIterHandle) -> MTPos;

    /// Add delta to pos.col of the key at the iterator position.
    #[allow(dead_code)]
    fn nvim_rawkey_add_pos_col(itr: MarkTreeIterHandle, delta: c_int);

    /// Add delta to pos.row of the key at the iterator position.
    #[allow(dead_code)]
    fn nvim_rawkey_add_pos_row(itr: MarkTreeIterHandle, delta: c_int);

    /// Allocate a zeroed MarkTreeIter on the heap.
    fn nvim_alloc_marktreeiter() -> MarkTreeIterHandle;

    /// Free a heap-allocated MarkTreeIter.
    fn nvim_free_marktreeiter(itr: MarkTreeIterHandle);

    /// Copy iterator contents from src to dst (equivalent to `*dst = *src`).
    fn nvim_marktree_itr_copy(dst: MarkTreeIterHandle, src: MarkTreeIterHandle);

    // ========================================================================
    // Memory Management Accessors (Phase 7)
    // ========================================================================

    /// Destroy the intersection kvec of a node (free heap storage).
    fn nvim_kvi_destroy_intersect(x: MTNodeHandle);

    /// Free a node's raw memory.
    fn nvim_xfree_node(x: MTNodeHandle);

    /// Decrement the node count on a marktree.
    fn nvim_marktree_dec_n_nodes(b: MarkTreeHandle);

    /// Set the number of keys in a marktree.
    fn nvim_marktree_set_n_keys(b: MarkTreeHandle, n: usize);

    /// Destroy the id2node hash map.
    fn nvim_marktree_destroy_id2node(b: MarkTreeHandle);
}

// ============================================================================
// Pure Helper Functions - Position Comparison
// ============================================================================

/// Check if position `a` is less than or equal to position `b`.
#[inline]
#[must_use]
pub const fn pos_leq(a: MTPos, b: MTPos) -> bool {
    a.row < b.row || (a.row == b.row && a.col <= b.col)
}

/// Check if position `a` is strictly less than position `b`.
#[inline]
#[must_use]
pub const fn pos_less(a: MTPos, b: MTPos) -> bool {
    !pos_leq(b, a)
}

/// Exported FFI version of `pos_leq`.
#[no_mangle]
pub extern "C" fn rs_pos_leq(a: MTPos, b: MTPos) -> bool {
    pos_leq(a, b)
}

/// Exported FFI version of `pos_less`.
#[no_mangle]
pub extern "C" fn rs_pos_less(a: MTPos, b: MTPos) -> bool {
    pos_less(a, b)
}

// ============================================================================
// Pure Helper Functions - Relative Positioning
// ============================================================================

/// Convert an absolute position to relative (to a base position).
///
/// After calling, `val` will be relative to `base`.
#[inline]
pub fn relative(base: MTPos, val: &mut MTPos) {
    debug_assert!(pos_leq(base, *val), "base must be <= val");
    if val.row == base.row {
        val.row = 0;
        val.col -= base.col;
    } else {
        val.row -= base.row;
    }
}

/// Convert a relative position to absolute (from a base position).
///
/// After calling, `val` will be absolute (based on `base`).
#[inline]
pub fn unrelative(base: MTPos, val: &mut MTPos) {
    if val.row == 0 {
        val.row = base.row;
        val.col += base.col;
    } else {
        val.row += base.row;
    }
}

/// Compose two relative positions.
///
/// Updates `base` by adding `val` to it.
#[inline]
pub fn compose(base: &mut MTPos, val: MTPos) {
    if val.row == 0 {
        base.col += val.col;
    } else {
        base.row += val.row;
        base.col = val.col;
    }
}

/// Exported FFI version of `relative`.
#[no_mangle]
pub extern "C" fn rs_relative(base: MTPos, val: *mut MTPos) {
    // SAFETY: Caller must provide valid pointer
    unsafe {
        if !val.is_null() {
            relative(base, &mut *val);
        }
    }
}

/// Exported FFI version of `unrelative`.
#[no_mangle]
pub extern "C" fn rs_unrelative(base: MTPos, val: *mut MTPos) {
    // SAFETY: Caller must provide valid pointer
    unsafe {
        if !val.is_null() {
            unrelative(base, &mut *val);
        }
    }
}

/// Exported FFI version of `compose`.
#[no_mangle]
pub extern "C" fn rs_compose(base: *mut MTPos, val: MTPos) {
    // SAFETY: Caller must provide valid pointer
    unsafe {
        if !base.is_null() {
            compose(&mut *base, val);
        }
    }
}

// ============================================================================
// Pure Helper Functions - ID Lookup
// ============================================================================

/// Compute the lookup ID for a mark.
///
/// The lookup ID combines namespace, id, and end flag into a single u64.
#[inline]
#[must_use]
pub const fn mt_lookup_id(ns: u32, id: u32, end: bool) -> u64 {
    ((ns as u64) << 33) | ((id as u64) << 1) | (if end { MARKTREE_END_FLAG } else { 0 })
}

/// Compute the lookup ID for a key, selecting start or end.
#[inline]
#[must_use]
pub const fn mt_lookup_key_side(key: &MTKey, end: bool) -> u64 {
    mt_lookup_id(key.ns, key.id, end)
}

/// Compute the lookup ID for a key based on its flags.
#[inline]
#[must_use]
pub const fn mt_lookup_key(key: &MTKey) -> u64 {
    mt_lookup_id(key.ns, key.id, key.flags & MT_FLAG_END != 0)
}

/// Exported FFI version of `mt_lookup_id`.
#[no_mangle]
pub extern "C" fn rs_mt_lookup_id(ns: u32, id: u32, end: bool) -> u64 {
    mt_lookup_id(ns, id, end)
}

/// Exported FFI version of `mt_lookup_key_side`.
#[no_mangle]
pub extern "C" fn rs_mt_lookup_key_side(key: MTKey, end: bool) -> u64 {
    mt_lookup_key_side(&key, end)
}

/// Exported FFI version of `mt_lookup_key`.
#[no_mangle]
pub extern "C" fn rs_mt_lookup_key(key: MTKey) -> u64 {
    mt_lookup_key(&key)
}

// ============================================================================
// Pure Helper Functions - Flag Checks
// ============================================================================

/// Check if a key is part of a paired mark.
#[inline]
#[must_use]
pub const fn mt_paired(key: &MTKey) -> bool {
    key.flags & MT_FLAG_PAIRED != 0
}

/// Check if a key is the end of a paired mark.
#[inline]
#[must_use]
pub const fn mt_end(key: &MTKey) -> bool {
    key.flags & MT_FLAG_END != 0
}

/// Check if a key is the start of a paired mark.
#[inline]
#[must_use]
pub const fn mt_start(key: &MTKey) -> bool {
    mt_paired(key) && !mt_end(key)
}

/// Check if a key has right gravity.
#[inline]
#[must_use]
pub const fn mt_right(key: &MTKey) -> bool {
    key.flags & MT_FLAG_RIGHT_GRAVITY != 0
}

/// Check if a key should not be undone.
#[inline]
#[must_use]
pub const fn mt_no_undo(key: &MTKey) -> bool {
    key.flags & MT_FLAG_NO_UNDO != 0
}

/// Check if a key can be invalidated.
#[inline]
#[must_use]
pub const fn mt_invalidate(key: &MTKey) -> bool {
    key.flags & MT_FLAG_INVALIDATE != 0
}

/// Check if a key is currently invalid.
#[inline]
#[must_use]
pub const fn mt_invalid(key: &MTKey) -> bool {
    key.flags & MT_FLAG_INVALID != 0
}

/// Check if a key has any decoration.
#[inline]
#[must_use]
pub const fn mt_decor_any(key: &MTKey) -> bool {
    key.flags & MT_FLAG_DECOR_MASK != 0
}

/// Check if a key has sign decoration.
#[inline]
#[must_use]
pub const fn mt_decor_sign(key: &MTKey) -> bool {
    key.flags & (MT_FLAG_DECOR_SIGNTEXT | MT_FLAG_DECOR_SIGNHL) != 0
}

/// Check if a key has concealed lines decoration.
#[inline]
#[must_use]
pub const fn mt_conceal_lines(key: &MTKey) -> bool {
    key.flags & MT_FLAG_DECOR_CONCEAL_LINES != 0
}

/// Exported FFI versions of flag checks.
#[no_mangle]
pub extern "C" fn rs_mt_paired(key: MTKey) -> bool {
    mt_paired(&key)
}

#[no_mangle]
pub extern "C" fn rs_mt_end(key: MTKey) -> bool {
    mt_end(&key)
}

#[no_mangle]
pub extern "C" fn rs_mt_start(key: MTKey) -> bool {
    mt_start(&key)
}

#[no_mangle]
pub extern "C" fn rs_mt_right(key: MTKey) -> bool {
    mt_right(&key)
}

#[no_mangle]
pub extern "C" fn rs_mt_no_undo(key: MTKey) -> bool {
    mt_no_undo(&key)
}

#[no_mangle]
pub extern "C" fn rs_mt_invalidate(key: MTKey) -> bool {
    mt_invalidate(&key)
}

#[no_mangle]
pub extern "C" fn rs_mt_invalid(key: MTKey) -> bool {
    mt_invalid(&key)
}

#[no_mangle]
pub extern "C" fn rs_mt_decor_any(key: MTKey) -> bool {
    mt_decor_any(&key)
}

#[no_mangle]
pub extern "C" fn rs_mt_decor_sign(key: MTKey) -> bool {
    mt_decor_sign(&key)
}

#[no_mangle]
pub extern "C" fn rs_mt_conceal_lines(key: MTKey) -> bool {
    mt_conceal_lines(&key)
}

// ============================================================================
// Pure Helper Functions - Key Comparison
// ============================================================================

/// Generic comparison helper for i32.
#[inline]
#[must_use]
const fn cmp_i32(a: i32, b: i32) -> c_int {
    if a < b {
        -1
    } else if a > b {
        1
    } else {
        0
    }
}

/// Generic comparison helper for u16.
#[inline]
#[must_use]
const fn cmp_u16(a: u16, b: u16) -> c_int {
    if a < b {
        -1
    } else if a > b {
        1
    } else {
        0
    }
}

/// Compare two keys for ordering in the tree.
///
/// Keys are ordered by:
/// 1. Row
/// 2. Column
/// 3. Flags (right gravity, end, real, last)
#[must_use]
pub fn key_cmp(a: &MTKey, b: &MTKey) -> c_int {
    // Compare row
    let cmp = cmp_i32(a.pos.row, b.pos.row);
    if cmp != 0 {
        return cmp;
    }

    // Compare column
    let cmp = cmp_i32(a.pos.col, b.pos.col);
    if cmp != 0 {
        return cmp;
    }

    // Compare flags (only comparison-relevant flags)
    const CMP_MASK: u16 = MT_FLAG_RIGHT_GRAVITY | MT_FLAG_END | MT_FLAG_REAL | MT_FLAG_LAST;
    cmp_u16(a.flags & CMP_MASK, b.flags & CMP_MASK)
}

/// Exported FFI version of `key_cmp`.
#[no_mangle]
pub extern "C" fn rs_key_cmp(a: MTKey, b: MTKey) -> c_int {
    key_cmp(&a, &b)
}

// ============================================================================
// Pure Helper Functions - Flag Construction
// ============================================================================

/// Construct flags from individual boolean values.
#[inline]
#[must_use]
pub const fn mt_flags(
    right_gravity: bool,
    no_undo: bool,
    invalidate: bool,
    decor_ext: bool,
) -> u16 {
    (if right_gravity {
        MT_FLAG_RIGHT_GRAVITY
    } else {
        0
    }) | (if no_undo { MT_FLAG_NO_UNDO } else { 0 })
        | (if invalidate { MT_FLAG_INVALIDATE } else { 0 })
        | (if decor_ext { MT_FLAG_DECOR_EXT } else { 0 })
}

/// Exported FFI version of `mt_flags`.
#[no_mangle]
pub extern "C" fn rs_mt_flags(
    right_gravity: bool,
    no_undo: bool,
    invalidate: bool,
    decor_ext: bool,
) -> u16 {
    mt_flags(right_gravity, no_undo, invalidate, decor_ext)
}

// ============================================================================
// Node Access Helpers (using C accessors)
// ============================================================================

/// Get the number of keys in a node.
#[inline]
#[must_use]
pub fn mtnode_n(x: MTNodeHandle) -> i32 {
    // SAFETY: C function is safe to call with valid handle
    unsafe { nvim_mtnode_get_n(x) }
}

/// Get the level of a node (0 for leaf).
#[inline]
#[must_use]
pub fn mtnode_level(x: MTNodeHandle) -> i32 {
    // SAFETY: C function is safe to call with valid handle
    unsafe { nvim_mtnode_get_level(x) }
}

/// Get a key from a node by index.
#[inline]
#[must_use]
pub fn mtnode_key(x: MTNodeHandle, idx: i32) -> MTKey {
    // SAFETY: C function is safe to call with valid handle and valid index
    unsafe { nvim_mtnode_get_key(x, idx) }
}

/// Get a child pointer from a node by index.
#[inline]
#[must_use]
pub fn mtnode_ptr(x: MTNodeHandle, idx: i32) -> MTNodeHandle {
    // SAFETY: C function is safe to call with valid handle and valid index
    unsafe { nvim_mtnode_get_ptr(x, idx) }
}

/// Get the root node of a marktree.
#[inline]
#[must_use]
pub fn marktree_root(b: MarkTreeHandle) -> MTNodeHandle {
    // SAFETY: C function is safe to call with valid handle
    unsafe { nvim_marktree_get_root(b) }
}

/// Get the total number of keys in a marktree.
#[inline]
#[must_use]
pub fn marktree_n_keys(b: MarkTreeHandle) -> usize {
    // SAFETY: C function is safe to call with valid handle
    unsafe { nvim_marktree_get_n_keys(b) }
}

// ============================================================================
// Binary Search Helper
// ============================================================================

/// Find position of key in node, or where it should be inserted.
///
/// Returns the position of `k` if it exists in the node, otherwise
/// the position it should be inserted (ranges from 0 to x->n inclusively).
///
/// If `found` is returned as `true`, an exact match was found.
#[must_use]
pub fn marktree_getp_aux(x: MTNodeHandle, k: &MTKey) -> (i32, bool) {
    let n = mtnode_n(x);
    if n == 0 {
        return (-1, false);
    }

    let mut begin = 0;
    let mut end = n;

    while begin < end {
        let mid = (begin + end) >> 1;
        let mid_key = mtnode_key(x, mid);
        if key_cmp(&mid_key, k) < 0 {
            begin = mid + 1;
        } else {
            end = mid;
        }
    }

    if begin == n {
        return (n - 1, false);
    }

    let begin_key = mtnode_key(x, begin);
    let found = key_cmp(k, &begin_key) == 0;
    if found {
        (begin, true)
    } else {
        (begin - 1, false)
    }
}

/// Exported FFI version of `marktree_getp_aux`.
///
/// Returns the position, sets `match_out` to true if exact match found.
#[no_mangle]
pub extern "C" fn rs_marktree_getp_aux(x: MTNodeHandle, k: MTKey, match_out: *mut bool) -> c_int {
    let (pos, found) = marktree_getp_aux(x, &k);
    // SAFETY: Caller must provide valid pointer or null
    unsafe {
        if !match_out.is_null() {
            *match_out = found;
        }
    }
    pos
}

// ============================================================================
// Iterator Helper Functions
// ============================================================================

/// Check if an iterator is valid (points to a mark).
#[inline]
#[must_use]
pub fn marktree_itr_valid(itr: MarkTreeIterHandle) -> bool {
    !unsafe { nvim_mtitr_get_x(itr) }.is_null()
}

/// Exported FFI version of `marktree_itr_valid`.
#[no_mangle]
pub extern "C" fn rs_marktree_itr_valid(itr: MarkTreeIterHandle) -> bool {
    marktree_itr_valid(itr)
}

/// Get the raw key at the current iterator position (without position adjustment).
#[inline]
#[must_use]
pub fn rawkey(itr: MarkTreeIterHandle) -> MTKey {
    let x = unsafe { nvim_mtitr_get_x(itr) };
    let i = unsafe { nvim_mtitr_get_i(itr) };
    mtnode_key(x, i)
}

/// Get the absolute position of the current iterator.
#[must_use]
pub fn marktree_itr_pos(itr: MarkTreeIterHandle) -> MTPos {
    let rkey = rawkey(itr);
    let base_pos = unsafe { nvim_mtitr_get_pos(itr) };
    let mut pos = rkey.pos;
    unrelative(base_pos, &mut pos);
    pos
}

/// Exported FFI version of `marktree_itr_pos`.
#[no_mangle]
pub extern "C" fn rs_marktree_itr_pos(itr: MarkTreeIterHandle) -> MTPos {
    marktree_itr_pos(itr)
}

/// Get the current mark from an iterator, with absolute position.
#[must_use]
pub fn marktree_itr_current(itr: MarkTreeIterHandle) -> MTKey {
    if !marktree_itr_valid(itr) {
        return MTKey::invalid();
    }
    let mut key = rawkey(itr);
    key.pos = marktree_itr_pos(itr);
    key
}

/// Exported FFI version of `marktree_itr_current`.
#[no_mangle]
pub extern "C" fn rs_marktree_itr_current(itr: MarkTreeIterHandle) -> MTKey {
    marktree_itr_current(itr)
}

/// Check if we're at the last key of the current node.
#[must_use]
pub fn marktree_itr_node_done(itr: MarkTreeIterHandle) -> bool {
    if !marktree_itr_valid(itr) {
        return true;
    }
    let x = unsafe { nvim_mtitr_get_x(itr) };
    let i = unsafe { nvim_mtitr_get_i(itr) };
    let n = mtnode_n(x);
    i == n - 1
}

/// Exported FFI version of `marktree_itr_node_done`.
#[no_mangle]
pub extern "C" fn rs_marktree_itr_node_done(itr: MarkTreeIterHandle) -> bool {
    marktree_itr_node_done(itr)
}

/// Move iterator to next mark.
///
/// Returns true if successful, false if we've reached the end.
#[must_use]
pub fn marktree_itr_next(_b: MarkTreeHandle, itr: MarkTreeIterHandle) -> bool {
    let x = unsafe { nvim_mtitr_get_x(itr) };
    if x.is_null() {
        return false;
    }

    let mut i = unsafe { nvim_mtitr_get_i(itr) };
    i += 1;
    unsafe { nvim_mtitr_set_i(itr, i) };

    let level = mtnode_level(x);
    let n = mtnode_n(x);

    if level == 0 {
        // At leaf node
        if i < n {
            return true;
        }
        // Go up until we find an internal key
        let mut current_x = x;
        let mut current_lvl = unsafe { nvim_mtitr_get_lvl(itr) };
        let mut current_pos = unsafe { nvim_mtitr_get_pos(itr) };

        while i >= mtnode_n(current_x) {
            let parent = unsafe { nvim_mtnode_get_parent(current_x) };
            if parent.is_null() {
                unsafe { nvim_mtitr_set_x(itr, MTNodeHandle::null()) };
                return false;
            }
            current_lvl -= 1;
            i = unsafe { nvim_mtitr_get_s_i(itr, current_lvl) };
            if i > 0 {
                let parent_key = mtnode_key(parent, i - 1);
                current_pos.row -= parent_key.pos.row;
                current_pos.col = unsafe { nvim_mtitr_get_s_oldcol(itr, current_lvl) };
            }
            current_x = parent;
        }
        unsafe {
            nvim_mtitr_set_x(itr, current_x);
            nvim_mtitr_set_i(itr, i);
            nvim_mtitr_set_lvl(itr, current_lvl);
            nvim_mtitr_set_pos(itr, current_pos);
        }
    } else {
        // At internal node - go down to first key
        let mut current_x = x;
        let mut current_i = i;
        let mut current_lvl = unsafe { nvim_mtitr_get_lvl(itr) };
        let mut current_pos = unsafe { nvim_mtitr_get_pos(itr) };

        while mtnode_level(current_x) > 0 {
            if current_i > 0 {
                let oldcol = current_pos.col;
                unsafe { nvim_mtitr_set_s_oldcol(itr, current_lvl, oldcol) };
                let key = mtnode_key(current_x, current_i - 1);
                compose(&mut current_pos, key.pos);
            }
            unsafe { nvim_mtitr_set_s_i(itr, current_lvl, current_i) };
            current_lvl += 1;
            current_x = mtnode_ptr(current_x, current_i);
            current_i = 0;
        }
        unsafe {
            nvim_mtitr_set_x(itr, current_x);
            nvim_mtitr_set_i(itr, current_i);
            nvim_mtitr_set_lvl(itr, current_lvl);
            nvim_mtitr_set_pos(itr, current_pos);
        }
    }
    true
}

/// Exported FFI version of `marktree_itr_next`.
#[no_mangle]
pub extern "C" fn rs_marktree_itr_next(b: MarkTreeHandle, itr: MarkTreeIterHandle) -> bool {
    marktree_itr_next(b, itr)
}

/// Move iterator to previous mark.
///
/// Returns true if successful, false if we've reached the beginning.
#[must_use]
pub fn marktree_itr_prev(_b: MarkTreeHandle, itr: MarkTreeIterHandle) -> bool {
    let x = unsafe { nvim_mtitr_get_x(itr) };
    if x.is_null() {
        return false;
    }

    let level = mtnode_level(x);
    let i = unsafe { nvim_mtitr_get_i(itr) };

    if level == 0 {
        // At leaf node
        let new_i = i - 1;
        unsafe { nvim_mtitr_set_i(itr, new_i) };
        if new_i >= 0 {
            return true;
        }
        // Go up until we find a non-internal key
        let mut current_x = x;
        let mut current_i = new_i;
        let mut current_lvl = unsafe { nvim_mtitr_get_lvl(itr) };
        let mut current_pos = unsafe { nvim_mtitr_get_pos(itr) };

        while current_i < 0 {
            let parent = unsafe { nvim_mtnode_get_parent(current_x) };
            if parent.is_null() {
                unsafe { nvim_mtitr_set_x(itr, MTNodeHandle::null()) };
                return false;
            }
            current_lvl -= 1;
            current_i = unsafe { nvim_mtitr_get_s_i(itr, current_lvl) } - 1;
            if current_i >= 0 {
                let parent_key = mtnode_key(parent, current_i);
                current_pos.row -= parent_key.pos.row;
                current_pos.col = unsafe { nvim_mtitr_get_s_oldcol(itr, current_lvl) };
            }
            current_x = parent;
        }
        unsafe {
            nvim_mtitr_set_x(itr, current_x);
            nvim_mtitr_set_i(itr, current_i);
            nvim_mtitr_set_lvl(itr, current_lvl);
            nvim_mtitr_set_pos(itr, current_pos);
        }
    } else {
        // At internal node - go down to last key
        let mut current_x = x;
        let mut current_i = i;
        let mut current_lvl = unsafe { nvim_mtitr_get_lvl(itr) };
        let mut current_pos = unsafe { nvim_mtitr_get_pos(itr) };

        while mtnode_level(current_x) > 0 {
            if current_i > 0 {
                let oldcol = current_pos.col;
                unsafe { nvim_mtitr_set_s_oldcol(itr, current_lvl, oldcol) };
                let key = mtnode_key(current_x, current_i - 1);
                compose(&mut current_pos, key.pos);
            }
            unsafe { nvim_mtitr_set_s_i(itr, current_lvl, current_i) };
            current_x = mtnode_ptr(current_x, current_i);
            current_i = mtnode_n(current_x);
            current_lvl += 1;
        }
        current_i -= 1;
        unsafe {
            nvim_mtitr_set_x(itr, current_x);
            nvim_mtitr_set_i(itr, current_i);
            nvim_mtitr_set_lvl(itr, current_lvl);
            nvim_mtitr_set_pos(itr, current_pos);
        }
    }
    true
}

/// Exported FFI version of `marktree_itr_prev`.
#[no_mangle]
pub extern "C" fn rs_marktree_itr_prev(b: MarkTreeHandle, itr: MarkTreeIterHandle) -> bool {
    marktree_itr_prev(b, itr)
}

/// Initialize iterator to the first mark in the tree.
///
/// Returns true if successful, false if tree is empty.
#[must_use]
pub fn marktree_itr_first(b: MarkTreeHandle, itr: MarkTreeIterHandle) -> bool {
    if marktree_n_keys(b) == 0 {
        unsafe { nvim_mtitr_set_x(itr, MTNodeHandle::null()) };
        return false;
    }

    let mut x = marktree_root(b);
    unsafe {
        nvim_mtitr_set_i(itr, 0);
        nvim_mtitr_set_lvl(itr, 0);
        nvim_mtitr_set_pos(itr, MTPos::new(0, 0));
    }

    let mut lvl = 0;
    while mtnode_level(x) > 0 {
        unsafe {
            nvim_mtitr_set_s_i(itr, lvl, 0);
            nvim_mtitr_set_s_oldcol(itr, lvl, 0);
        }
        lvl += 1;
        x = mtnode_ptr(x, 0);
    }
    unsafe {
        nvim_mtitr_set_x(itr, x);
        nvim_mtitr_set_lvl(itr, lvl);
    }
    true
}

/// Exported FFI version of `marktree_itr_first`.
#[no_mangle]
pub extern "C" fn rs_marktree_itr_first(b: MarkTreeHandle, itr: MarkTreeIterHandle) -> bool {
    marktree_itr_first(b, itr)
}

/// Position iterator at a given row/col.
///
/// Returns true if successful, false if tree is empty.
/// The iterator will point to the first mark at or after the given position.
#[must_use]
pub fn marktree_itr_get(b: MarkTreeHandle, row: i32, col: i32, itr: MarkTreeIterHandle) -> bool {
    marktree_itr_get_ext(b, MTPos::new(row, col), itr, false, false)
}

/// Exported FFI version of `marktree_itr_get`.
#[no_mangle]
pub extern "C" fn rs_marktree_itr_get(
    b: MarkTreeHandle,
    row: i32,
    col: i32,
    itr: MarkTreeIterHandle,
) -> bool {
    marktree_itr_get(b, row, col, itr)
}

/// Extended version of iterator positioning with full parameters.
///
/// If `last` is true, position at the last key <= position.
/// If `gravity` is true, consider right gravity when positioning.
/// If `oldbase` is non-null, records base positions at each level.
/// If `meta_filter` is non-null, skips subtrees that don't match the filter.
#[allow(clippy::many_single_char_names)] // Matching C code naming conventions
#[allow(clippy::cast_sign_loss)] // Level values are always non-negative
#[must_use]
pub fn marktree_itr_get_ext_full(
    b: MarkTreeHandle,
    p: MTPos,
    itr: MarkTreeIterHandle,
    last: bool,
    gravity: bool,
    oldbase: *mut MTPos,
    meta_filter: MetaFilter,
) -> bool {
    if marktree_n_keys(b) == 0 {
        unsafe { nvim_mtitr_set_x(itr, MTNodeHandle::null()) };
        return false;
    }

    // Create search key with appropriate flags
    let flags = if gravity {
        MT_FLAG_RIGHT_GRAVITY
    } else if last {
        MT_FLAG_LAST
    } else {
        0
    };
    let mut k = MTKey {
        pos: p,
        ns: 0,
        id: 0,
        flags,
        decor_data: DecorInlineData::zero(),
    };

    unsafe {
        nvim_mtitr_set_pos(itr, MTPos::new(0, 0));
        nvim_mtitr_set_lvl(itr, 0);
    }

    let mut x = marktree_root(b);
    let mut current_pos = MTPos::new(0, 0);
    let mut lvl = 0;

    if !oldbase.is_null() {
        unsafe { *oldbase.add(lvl as usize) = current_pos };
    }

    loop {
        let (getp_i, _) = marktree_getp_aux(x, &k);
        let i = getp_i + 1; // marktree_getp_aux returns position before, we want after

        if mtnode_level(x) == 0 {
            unsafe {
                nvim_mtitr_set_x(itr, x);
                nvim_mtitr_set_i(itr, i);
                nvim_mtitr_set_lvl(itr, lvl);
                nvim_mtitr_set_pos(itr, current_pos);
            }
            break;
        }

        if !meta_filter.is_null() {
            let meta = mtnode_meta(x, i);
            if !meta_has(&meta, meta_filter) {
                // This takes us to the internal position after the first rejected node
                unsafe {
                    nvim_mtitr_set_x(itr, x);
                    nvim_mtitr_set_i(itr, i);
                    nvim_mtitr_set_lvl(itr, lvl);
                    nvim_mtitr_set_pos(itr, current_pos);
                }
                break;
            }
        }

        unsafe {
            nvim_mtitr_set_s_i(itr, lvl, i);
            nvim_mtitr_set_s_oldcol(itr, lvl, current_pos.col);
        }

        if i > 0 {
            let key_pos = mtnode_key(x, i - 1).pos;
            compose(&mut current_pos, key_pos);
            relative(key_pos, &mut k.pos);
        }
        x = mtnode_ptr(x, i);
        lvl += 1;

        if !oldbase.is_null() {
            unsafe { *oldbase.add(lvl as usize) = current_pos };
        }
    }

    if last {
        marktree_itr_prev(b, itr)
    } else {
        let i = unsafe { nvim_mtitr_get_i(itr) };
        let x = unsafe { nvim_mtitr_get_x(itr) };
        if i >= mtnode_n(x) {
            // No need for "meta_filter" here, this just goes up one step
            marktree_itr_next_skip(b, itr, true, false, std::ptr::null_mut(), std::ptr::null())
        } else {
            true
        }
    }
}

/// Simple version without oldbase/meta_filter (convenience wrapper).
#[must_use]
pub fn marktree_itr_get_ext(
    b: MarkTreeHandle,
    p: MTPos,
    itr: MarkTreeIterHandle,
    last: bool,
    gravity: bool,
) -> bool {
    marktree_itr_get_ext_full(
        b,
        p,
        itr,
        last,
        gravity,
        std::ptr::null_mut(),
        std::ptr::null(),
    )
}

/// Exported FFI version of `marktree_itr_get_ext` (simple, no oldbase/meta_filter).
#[no_mangle]
pub extern "C" fn rs_marktree_itr_get_ext(
    b: MarkTreeHandle,
    p: MTPos,
    itr: MarkTreeIterHandle,
    last: bool,
    gravity: bool,
) -> bool {
    marktree_itr_get_ext(b, p, itr, last, gravity)
}

/// Exported FFI version of `marktree_itr_get_ext` with full parameters.
#[no_mangle]
pub extern "C" fn rs_marktree_itr_get_ext_full(
    b: MarkTreeHandle,
    p: MTPos,
    itr: MarkTreeIterHandle,
    last: bool,
    gravity: bool,
    oldbase: *mut MTPos,
    meta_filter: MetaFilter,
) -> bool {
    marktree_itr_get_ext_full(b, p, itr, last, gravity, oldbase, meta_filter)
}

// ============================================================================
// Lookup and Pair Functions
// ============================================================================

/// Lookup a mark by its combined lookup ID.
///
/// Returns the mark with absolute position, or an invalid key if not found.
/// If `itr` is not null, positions the iterator at the found mark.
#[must_use]
pub fn marktree_lookup(b: MarkTreeHandle, id: u64, itr: MarkTreeIterHandle) -> MTKey {
    let n = marktree_id2node(b, id);
    if n.is_null() {
        if !itr.is_null() {
            unsafe { nvim_mtitr_set_x(itr, MTNodeHandle::null()) };
        }
        return MTKey::invalid();
    }
    let n_keys = mtnode_n(n);
    for i in 0..n_keys {
        let key = mtnode_key(n, i);
        if mt_lookup_key(&key) == id {
            return marktree_itr_set_node(b, itr, n, i);
        }
    }
    // Should not reach here if id2node returned a valid node
    // In C this calls abort(), but we use the same pattern
    unreachable!("marktree_lookup: id found in id2node but not in node keys");
}

/// Exported FFI version of `marktree_lookup`.
#[no_mangle]
pub extern "C" fn rs_marktree_lookup(b: MarkTreeHandle, id: u64, itr: MarkTreeIterHandle) -> MTKey {
    marktree_lookup(b, id, itr)
}

/// Lookup a mark by namespace and ID.
///
/// If `end` is true, looks up the end mark; otherwise the start mark.
#[must_use]
pub fn marktree_lookup_ns(
    b: MarkTreeHandle,
    ns: u32,
    id: u32,
    end: bool,
    itr: MarkTreeIterHandle,
) -> MTKey {
    marktree_lookup(b, mt_lookup_id(ns, id, end), itr)
}

/// Exported FFI version of `marktree_lookup_ns`.
#[no_mangle]
pub extern "C" fn rs_marktree_lookup_ns(
    b: MarkTreeHandle,
    ns: u32,
    id: u32,
    end: bool,
    itr: MarkTreeIterHandle,
) -> MTKey {
    marktree_lookup_ns(b, ns, id, end, itr)
}

/// Get the alternate end of a paired mark.
///
/// For a paired mark, returns the other end (start->end or end->start).
/// For an unpaired mark, returns the mark itself.
#[must_use]
pub fn marktree_get_alt(b: MarkTreeHandle, mark: MTKey, itr: MarkTreeIterHandle) -> MTKey {
    if mt_paired(&mark) {
        marktree_lookup_ns(b, mark.ns, mark.id, !mt_end(&mark), itr)
    } else {
        mark
    }
}

/// Exported FFI version of `marktree_get_alt`.
#[no_mangle]
pub extern "C" fn rs_marktree_get_alt(
    b: MarkTreeHandle,
    mark: MTKey,
    itr: MarkTreeIterHandle,
) -> MTKey {
    marktree_get_alt(b, mark, itr)
}

/// Get the position of the alternate end of a paired mark.
///
/// Convenience function that just returns the position.
#[must_use]
pub fn marktree_get_altpos(b: MarkTreeHandle, mark: MTKey, itr: MarkTreeIterHandle) -> MTPos {
    marktree_get_alt(b, mark, itr).pos
}

/// Exported FFI version of `marktree_get_altpos`.
#[no_mangle]
pub extern "C" fn rs_marktree_get_altpos(
    b: MarkTreeHandle,
    mark: MTKey,
    itr: MarkTreeIterHandle,
) -> MTPos {
    marktree_get_altpos(b, mark, itr)
}

// ============================================================================
// Meta Count Helpers
// ============================================================================

/// Meta index values (matching C MetaIndex enum).
pub mod meta_index {
    pub const K_MT_META_INLINE: usize = 0;
    pub const K_MT_META_LINES: usize = 1;
    pub const K_MT_META_SIGN_HL: usize = 2;
    pub const K_MT_META_SIGN_TEXT: usize = 3;
    pub const K_MT_META_CONCEAL_LINES: usize = 4;
    pub const K_MT_META_COUNT: usize = 5;
}

use meta_index::*;

/// Compute the meta flags for a key.
///
/// Returns an array of counts (0 or 1) for each meta category.
#[must_use]
pub fn meta_describe_key(k: &MTKey) -> [u32; K_MT_META_COUNT] {
    let mut meta_inc = [0u32; K_MT_META_COUNT];

    // Don't count end keys or invalid keys
    if mt_end(k) || mt_invalid(k) {
        return meta_inc;
    }

    if k.flags & MT_FLAG_DECOR_VIRT_TEXT_INLINE != 0 {
        meta_inc[K_MT_META_INLINE] = 1;
    }
    if k.flags & flags::MT_FLAG_DECOR_VIRT_LINES != 0 {
        meta_inc[K_MT_META_LINES] = 1;
    }
    if k.flags & MT_FLAG_DECOR_SIGNHL != 0 {
        meta_inc[K_MT_META_SIGN_HL] = 1;
    }
    if k.flags & MT_FLAG_DECOR_SIGNTEXT != 0 {
        meta_inc[K_MT_META_SIGN_TEXT] = 1;
    }
    if k.flags & MT_FLAG_DECOR_CONCEAL_LINES != 0 {
        meta_inc[K_MT_META_CONCEAL_LINES] = 1;
    }

    meta_inc
}

/// Exported FFI version of `meta_describe_key`.
///
/// Writes the meta counts to the provided array.
#[no_mangle]
pub extern "C" fn rs_meta_describe_key(k: MTKey, meta_inc: *mut u32) {
    let result = meta_describe_key(&k);
    unsafe {
        if !meta_inc.is_null() {
            for (i, &val) in result.iter().enumerate() {
                *meta_inc.add(i) = val;
            }
        }
    }
}

// ============================================================================
// Phase 3: Meta and Helper Functions
// ============================================================================

/// Increment meta counts for a key.
///
/// This adds to existing counts rather than replacing them.
pub fn meta_describe_key_inc(meta_inc: &mut [u32; K_MT_META_COUNT], k: &MTKey) {
    if mt_end(k) || mt_invalid(k) {
        return;
    }

    if k.flags & MT_FLAG_DECOR_VIRT_TEXT_INLINE != 0 {
        meta_inc[K_MT_META_INLINE] += 1;
    }
    if k.flags & flags::MT_FLAG_DECOR_VIRT_LINES != 0 {
        meta_inc[K_MT_META_LINES] += 1;
    }
    if k.flags & MT_FLAG_DECOR_SIGNHL != 0 {
        meta_inc[K_MT_META_SIGN_HL] += 1;
    }
    if k.flags & MT_FLAG_DECOR_SIGNTEXT != 0 {
        meta_inc[K_MT_META_SIGN_TEXT] += 1;
    }
    if k.flags & MT_FLAG_DECOR_CONCEAL_LINES != 0 {
        meta_inc[K_MT_META_CONCEAL_LINES] += 1;
    }
}

/// Exported FFI version of `meta_describe_key_inc`.
#[no_mangle]
pub extern "C" fn rs_meta_describe_key_inc(meta_inc: *mut u32, k: *mut MTKey) {
    unsafe {
        if !meta_inc.is_null() && !k.is_null() {
            let mut meta = [0u32; K_MT_META_COUNT];
            for (i, m) in meta.iter_mut().enumerate() {
                *m = *meta_inc.add(i);
            }
            meta_describe_key_inc(&mut meta, &*k);
            for (i, m) in meta.iter().enumerate() {
                *meta_inc.add(i) = *m;
            }
        }
    }
}

/// Compute meta counts for a node (sum over keys and children).
///
/// For internal nodes, also adds the meta counts from all children.
/// Ported from C `meta_describe_node`.
#[must_use]
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
pub fn meta_describe_node(x: MTNodeHandle) -> [u32; K_MT_META_COUNT] {
    let n = unsafe { nvim_mtnode_get_n(x) };
    let level = unsafe { nvim_mtnode_get_level(x) };
    let mut meta_node = [0u32; K_MT_META_COUNT];
    for i in 0..n {
        let k = unsafe { nvim_mtnode_get_key(x, i) };
        meta_describe_key_inc(&mut meta_node, &k);
    }
    if level != 0 {
        for i in 0..=(n as usize) {
            for (m, slot) in meta_node.iter_mut().enumerate() {
                *slot += unsafe { nvim_mtnode_get_meta(x, i as c_int, m as c_int) };
            }
        }
    }
    meta_node
}

/// Exported FFI version of `meta_describe_node`.
#[no_mangle]
pub extern "C" fn rs_meta_describe_node(meta_out: *mut u32, x: MTNodeHandle) {
    let result = meta_describe_node(x);
    unsafe {
        if !meta_out.is_null() {
            for (i, &val) in result.iter().enumerate() {
                *meta_out.add(i) = val;
            }
        }
    }
}

/// Compute pseudo-index for a position in the tree.
///
/// Pseudo-indices allow efficient ordering comparisons between positions
/// without traversing the tree. They encode the path from root to the position.
/// A valid pseudo-index is never zero; zero is reserved as a sentinel for "not found".
#[must_use]
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap
)]
pub fn pseudo_index(x: MTNodeHandle, mut i: i32) -> u64 {
    let initial_level = unsafe { nvim_mtnode_get_level(x) };
    let mut off: u32 = (MT_LOG2_BRANCH as u32) * (initial_level as u32);
    let mut index: u64 = 0;
    let mut cur = x;
    while !cur.is_null() {
        index |= ((i as u64).wrapping_add(1)) << off;
        off += MT_LOG2_BRANCH as u32;
        i = unsafe { nvim_mtnode_get_p_idx(cur) };
        cur = unsafe { nvim_mtnode_get_parent(cur) };
    }
    index
}

/// Exported FFI version of `pseudo_index`.
#[no_mangle]
pub extern "C" fn rs_pseudo_index(x: MTNodeHandle, i: c_int) -> u64 {
    pseudo_index(x, i)
}

/// Compute pseudo-index for a mark ID.
///
/// If `sloppy` is true, all keys in the same leaf node get the same index.
#[must_use]
pub fn pseudo_index_for_id(b: MarkTreeHandle, id: u64, sloppy: bool) -> u64 {
    let n = marktree_id2node(b, id);
    if n.is_null() {
        return 0; // a valid pseudo-index is never zero!
    }

    let mut i = 0i32;
    let level = mtnode_level(n);
    let n_keys = mtnode_n(n);
    if level != 0 || !sloppy {
        while i < n_keys {
            let key = mtnode_key(n, i);
            if mt_lookup_key(&key) == id {
                break;
            }
            i += 1;
        }
        // assert: i < n_keys (invariant maintained by caller)
        if level != 0 {
            i += 1; // internal key i comes after ptr[i]
        }
    }

    pseudo_index(n, i)
}

/// Exported FFI version of `pseudo_index_for_id`.
#[no_mangle]
pub extern "C" fn rs_pseudo_index_for_id(b: MarkTreeHandle, id: u64, sloppy: bool) -> u64 {
    pseudo_index_for_id(b, id, sloppy)
}

/// Set iterator to point at a specific node and index.
///
/// Walks up from node `n` to the root, unrelativizing the key position
/// and recording the path in the iterator's stack. Returns the key with
/// absolute position.
#[must_use]
pub fn marktree_itr_set_node(
    b: MarkTreeHandle,
    itr: MarkTreeIterHandle,
    n: MTNodeHandle,
    i: i32,
) -> MTKey {
    let mut key = mtnode_key(n, i);
    let root_level = mtnode_level(marktree_root(b));

    if !itr.is_null() {
        unsafe {
            nvim_mtitr_set_i(itr, i);
            nvim_mtitr_set_x(itr, n);
            nvim_mtitr_set_lvl(itr, root_level - mtnode_level(n));
        }
    }

    // Walk up from n to root, unrelativizing position
    let mut current = n;
    loop {
        let parent = unsafe { nvim_mtnode_get_parent(current) };
        if parent.is_null() {
            break;
        }
        let current_i = unsafe { nvim_mtnode_get_p_idx(current) };

        if !itr.is_null() {
            let s_lvl = root_level - mtnode_level(parent);
            unsafe { nvim_mtitr_set_s_i(itr, s_lvl, current_i) };
        }

        if current_i > 0 {
            let parent_key_pos = mtnode_key(parent, current_i - 1).pos;
            unrelative(parent_key_pos, &mut key.pos);
        }
        current = parent;
    }

    if !itr.is_null() {
        marktree_itr_fix_pos(b, itr);
    }
    key
}

/// Exported FFI version of `marktree_itr_set_node`.
#[no_mangle]
pub extern "C" fn rs_marktree_itr_set_node(
    b: MarkTreeHandle,
    itr: MarkTreeIterHandle,
    n: MTNodeHandle,
    i: c_int,
) -> MTKey {
    marktree_itr_set_node(b, itr, n, i)
}

/// Fix iterator position after setting node directly.
///
/// Walks from root down to the iterator's current level, computing the
/// absolute position by composing key positions along the path.
pub fn marktree_itr_fix_pos(b: MarkTreeHandle, itr: MarkTreeIterHandle) {
    let mut pos = MTPos::new(0, 0);
    let mut x = marktree_root(b);
    let lvl = unsafe { nvim_mtitr_get_lvl(itr) };

    for l in 0..lvl {
        let oldcol = pos.col;
        unsafe { nvim_mtitr_set_s_oldcol(itr, l, oldcol) };
        let i = unsafe { nvim_mtitr_get_s_i(itr, l) };
        if i > 0 {
            let key_pos = mtnode_key(x, i - 1).pos;
            compose(&mut pos, key_pos);
        }
        debug_assert!(mtnode_level(x) > 0);
        x = mtnode_ptr(x, i);
    }
    // x should now be itr->x
    unsafe { nvim_mtitr_set_pos(itr, pos) };
}

/// Exported FFI version of `marktree_itr_fix_pos`.
#[no_mangle]
pub extern "C" fn rs_marktree_itr_fix_pos(b: MarkTreeHandle, itr: MarkTreeIterHandle) {
    marktree_itr_fix_pos(b, itr);
}

/// Lookup node by mark ID.
#[must_use]
pub fn marktree_id2node(b: MarkTreeHandle, id: u64) -> MTNodeHandle {
    unsafe { nvim_marktree_id2node(b, id) }
}

/// Exported FFI version of `marktree_id2node`.
#[no_mangle]
pub extern "C" fn rs_marktree_id2node(b: MarkTreeHandle, id: u64) -> MTNodeHandle {
    marktree_id2node(b, id)
}

// ============================================================================
// MTPair Type for Overlap Iteration
// ============================================================================

/// A pair of marks (start and end).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MTPair {
    pub start: MTKey,
    pub end_pos: MTPos,
    pub end_right_gravity: bool,
}

impl MTPair {
    /// Create a zero-initialized pair.
    #[inline]
    #[must_use]
    pub const fn zero() -> Self {
        Self {
            start: MTKey::zero(),
            end_pos: MTPos::new(0, 0),
            end_right_gravity: false,
        }
    }

    /// Create a pair from start and end keys.
    #[inline]
    #[must_use]
    pub fn from_keys(start: MTKey, end: MTKey) -> Self {
        Self {
            start,
            end_pos: end.pos,
            end_right_gravity: mt_right(&end),
        }
    }
}

// ============================================================================
// Phase 1: Iterator Completion Functions
// ============================================================================

/// Initialize iterator to the last mark in the tree.
///
/// Returns true if successful, false if tree is empty.
#[must_use]
pub fn marktree_itr_last(b: MarkTreeHandle, itr: MarkTreeIterHandle) -> bool {
    if marktree_n_keys(b) == 0 {
        unsafe { nvim_mtitr_set_x(itr, MTNodeHandle::null()) };
        return false;
    }

    let mut x = marktree_root(b);
    unsafe {
        nvim_mtitr_set_pos(itr, MTPos::new(0, 0));
        nvim_mtitr_set_lvl(itr, 0);
    }

    let mut lvl = 0;
    let mut current_pos = MTPos::new(0, 0);

    loop {
        let n = mtnode_n(x);
        unsafe { nvim_mtitr_set_i(itr, n) };

        if mtnode_level(x) == 0 {
            break;
        }

        unsafe {
            nvim_mtitr_set_s_i(itr, lvl, n);
            nvim_mtitr_set_s_oldcol(itr, lvl, current_pos.col);
        }

        // i > 0 always since n > 0 for non-empty tree
        let key = mtnode_key(x, n - 1);
        compose(&mut current_pos, key.pos);

        x = mtnode_ptr(x, n);
        lvl += 1;
    }

    let i = unsafe { nvim_mtitr_get_i(itr) };
    unsafe {
        nvim_mtitr_set_i(itr, i - 1);
        nvim_mtitr_set_x(itr, x);
        nvim_mtitr_set_lvl(itr, lvl);
        nvim_mtitr_set_pos(itr, current_pos);
    }
    true
}

/// Exported FFI version of `marktree_itr_last`.
#[no_mangle]
pub extern "C" fn rs_marktree_itr_last(b: MarkTreeHandle, itr: MarkTreeIterHandle) -> bool {
    marktree_itr_last(b, itr)
}

/// Helper type for meta filter (array of 5 u32s).
pub type MetaFilter = *const u32;

/// Check if meta counts match the filter.
#[inline]
#[must_use]
pub fn meta_has(meta_count: &[u32; K_MT_META_COUNT], meta_filter: MetaFilter) -> bool {
    if meta_filter.is_null() {
        return true;
    }
    unsafe { nvim_meta_has(meta_count.as_ptr(), meta_filter) }
}

/// Get meta counts from a node at a given index.
#[inline]
#[must_use]
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
pub fn mtnode_meta(x: MTNodeHandle, idx: i32) -> [u32; K_MT_META_COUNT] {
    let mut meta = [0u32; K_MT_META_COUNT];
    for (m, meta_item) in meta.iter_mut().enumerate() {
        *meta_item = unsafe { nvim_mtnode_get_meta(x, idx, m as c_int) };
    }
    meta
}

/// Get meta_root from a marktree.
#[inline]
#[must_use]
pub fn marktree_meta_root(b: MarkTreeHandle) -> [u32; K_MT_META_COUNT] {
    let mut meta = [0u32; K_MT_META_COUNT];
    unsafe { nvim_marktree_get_meta_root(b, meta.as_mut_ptr()) };
    meta
}

/// Internal advance with skip and meta filtering.
///
/// If `skip` is true, skips the subtree rooted at current position.
/// If `meta_filter` is non-null, uses it to filter which subtrees to enter.
/// If `oldbase` is non-null, updates the base position array for each level traversed.
#[allow(clippy::many_single_char_names)]
#[allow(clippy::branches_sharing_code)] // Different branches have different control flow
#[allow(clippy::cast_sign_loss)] // Level values are always non-negative
#[must_use]
pub fn marktree_itr_next_skip(
    _b: MarkTreeHandle,
    itr: MarkTreeIterHandle,
    mut skip: bool,
    preload: bool,
    oldbase: *mut MTPos,
    meta_filter: MetaFilter,
) -> bool {
    let x = unsafe { nvim_mtitr_get_x(itr) };
    if x.is_null() {
        return false;
    }

    let mut i = unsafe { nvim_mtitr_get_i(itr) };
    i += 1;
    unsafe { nvim_mtitr_set_i(itr, i) };

    let level = mtnode_level(x);

    // Check meta filter for internal nodes
    if !meta_filter.is_null() && level > 0 {
        let meta = mtnode_meta(x, i);
        if !meta_has(&meta, meta_filter) {
            skip = true;
        }
    }

    if level == 0 || skip {
        let mut current_x = x;
        let mut current_i = i;
        let n = mtnode_n(current_x);

        if preload && level == 0 && skip {
            // Skip rest of this leaf node
            current_i = n;
            unsafe { nvim_mtitr_set_i(itr, current_i) };
        } else if current_i < n {
            return true;
        }

        // Go up until we find an internal key
        let mut current_lvl = unsafe { nvim_mtitr_get_lvl(itr) };
        let mut current_pos = unsafe { nvim_mtitr_get_pos(itr) };

        while current_i >= mtnode_n(current_x) {
            let parent = unsafe { nvim_mtnode_get_parent(current_x) };
            if parent.is_null() {
                unsafe { nvim_mtitr_set_x(itr, MTNodeHandle::null()) };
                return false;
            }
            current_lvl -= 1;
            current_i = unsafe { nvim_mtitr_get_s_i(itr, current_lvl) };
            if current_i > 0 {
                let parent_key = mtnode_key(parent, current_i - 1);
                current_pos.row -= parent_key.pos.row;
                current_pos.col = unsafe { nvim_mtitr_get_s_oldcol(itr, current_lvl) };
            }
            current_x = parent;
        }
        unsafe {
            nvim_mtitr_set_x(itr, current_x);
            nvim_mtitr_set_i(itr, current_i);
            nvim_mtitr_set_lvl(itr, current_lvl);
            nvim_mtitr_set_pos(itr, current_pos);
        }
    } else {
        // At internal node - go down to first key
        let mut current_x = x;
        let mut current_i = i;
        let mut current_lvl = unsafe { nvim_mtitr_get_lvl(itr) };
        let mut current_pos = unsafe { nvim_mtitr_get_pos(itr) };

        while mtnode_level(current_x) > 0 {
            if current_i > 0 {
                let oldcol = current_pos.col;
                unsafe { nvim_mtitr_set_s_oldcol(itr, current_lvl, oldcol) };
                let key = mtnode_key(current_x, current_i - 1);
                compose(&mut current_pos, key.pos);
            }
            if !oldbase.is_null() && current_i == 0 {
                unsafe {
                    *oldbase.add((current_lvl + 1) as usize) = *oldbase.add(current_lvl as usize);
                }
            }
            unsafe { nvim_mtitr_set_s_i(itr, current_lvl, current_i) };
            current_lvl += 1;
            current_x = mtnode_ptr(current_x, current_i);

            if preload && mtnode_level(current_x) > 0 {
                current_i = -1;
                break;
            }
            current_i = 0;

            // Check meta filter
            if !meta_filter.is_null() && mtnode_level(current_x) > 0 {
                let meta = mtnode_meta(current_x, 0);
                if !meta_has(&meta, meta_filter) {
                    break;
                }
            }
        }
        unsafe {
            nvim_mtitr_set_x(itr, current_x);
            nvim_mtitr_set_i(itr, current_i);
            nvim_mtitr_set_lvl(itr, current_lvl);
            nvim_mtitr_set_pos(itr, current_pos);
        }
    }
    true
}

/// Exported FFI version of `marktree_itr_next_skip`.
#[no_mangle]
pub extern "C" fn rs_marktree_itr_next_skip(
    b: MarkTreeHandle,
    itr: MarkTreeIterHandle,
    skip: bool,
    preload: bool,
    oldbase: *mut MTPos,
    meta_filter: MetaFilter,
) -> bool {
    marktree_itr_next_skip(b, itr, skip, preload, oldbase, meta_filter)
}

/// Meta map for converting meta index to flag.
const META_MAP: [u32; K_MT_META_COUNT] = [
    MT_FLAG_DECOR_VIRT_TEXT_INLINE as u32,
    MT_FLAG_DECOR_VIRT_LINES as u32,
    MT_FLAG_DECOR_SIGNHL as u32,
    MT_FLAG_DECOR_SIGNTEXT as u32,
    MT_FLAG_DECOR_CONCEAL_LINES as u32,
];

/// Check if iterator position matches filter and is within bounds.
fn marktree_itr_check_filter(
    b: MarkTreeHandle,
    itr: MarkTreeIterHandle,
    stop_row: i32,
    stop_col: i32,
    meta_filter: MetaFilter,
) -> bool {
    let stop_pos = MTPos::new(stop_row, stop_col);

    // Build key filter from meta filter
    let mut key_filter: u32 = 0;
    if !meta_filter.is_null() {
        for (m, &flag) in META_MAP.iter().enumerate() {
            let filter_val = unsafe { *meta_filter.add(m) };
            key_filter |= flag & filter_val;
        }
    }

    loop {
        if pos_leq(stop_pos, marktree_itr_pos(itr)) {
            unsafe { nvim_mtitr_set_x(itr, MTNodeHandle::null()) };
            return false;
        }

        let k = rawkey(itr);
        if !mt_end(&k) && (u32::from(k.flags) & key_filter) != 0 {
            return true;
        }

        // Skip subtrees but not keys
        if !marktree_itr_next_skip(b, itr, false, false, std::ptr::null_mut(), meta_filter) {
            return false;
        }
    }
}

/// Exported FFI version of `marktree_itr_check_filter`.
#[no_mangle]
pub extern "C" fn rs_marktree_itr_check_filter(
    b: MarkTreeHandle,
    itr: MarkTreeIterHandle,
    stop_row: i32,
    stop_col: i32,
    meta_filter: MetaFilter,
) -> bool {
    marktree_itr_check_filter(b, itr, stop_row, stop_col, meta_filter)
}

/// Position iterator at a given position with meta filtering.
///
/// Returns true if a matching mark was found before the stop position.
#[must_use]
pub fn marktree_itr_get_filter(
    b: MarkTreeHandle,
    row: i32,
    col: i32,
    stop_row: i32,
    stop_col: i32,
    meta_filter: MetaFilter,
    itr: MarkTreeIterHandle,
) -> bool {
    // Check if tree has any matching marks
    let meta_root = marktree_meta_root(b);
    if !meta_has(&meta_root, meta_filter) {
        return false;
    }

    if !marktree_itr_get_ext(b, MTPos::new(row, col), itr, false, false) {
        return false;
    }

    marktree_itr_check_filter(b, itr, stop_row, stop_col, meta_filter)
}

/// Exported FFI version of `marktree_itr_get_filter`.
#[no_mangle]
pub extern "C" fn rs_marktree_itr_get_filter(
    b: MarkTreeHandle,
    row: i32,
    col: i32,
    stop_row: i32,
    stop_col: i32,
    meta_filter: MetaFilter,
    itr: MarkTreeIterHandle,
) -> bool {
    marktree_itr_get_filter(b, row, col, stop_row, stop_col, meta_filter, itr)
}

/// Move to next filtered mark.
///
/// Returns true if a matching mark was found before the stop position.
#[must_use]
pub fn marktree_itr_next_filter(
    b: MarkTreeHandle,
    itr: MarkTreeIterHandle,
    stop_row: i32,
    stop_col: i32,
    meta_filter: MetaFilter,
) -> bool {
    if !marktree_itr_next_skip(b, itr, false, false, std::ptr::null_mut(), meta_filter) {
        return false;
    }

    marktree_itr_check_filter(b, itr, stop_row, stop_col, meta_filter)
}

/// Exported FFI version of `marktree_itr_next_filter`.
#[no_mangle]
pub extern "C" fn rs_marktree_itr_next_filter(
    b: MarkTreeHandle,
    itr: MarkTreeIterHandle,
    stop_row: i32,
    stop_col: i32,
    meta_filter: MetaFilter,
) -> bool {
    marktree_itr_next_filter(b, itr, stop_row, stop_col, meta_filter)
}

/// Step out to parent that has matching filter.
///
/// Used after `marktree_itr_get_overlap()` to continue in filtered fashion.
#[must_use]
pub fn marktree_itr_step_out_filter(
    b: MarkTreeHandle,
    itr: MarkTreeIterHandle,
    meta_filter: MetaFilter,
) -> bool {
    let meta_root = marktree_meta_root(b);
    if !meta_has(&meta_root, meta_filter) {
        unsafe { nvim_mtitr_set_x(itr, MTNodeHandle::null()) };
        return false;
    }

    loop {
        let x = unsafe { nvim_mtitr_get_x(itr) };
        if x.is_null() {
            break;
        }

        let parent = unsafe { nvim_mtnode_get_parent(x) };
        if parent.is_null() {
            break;
        }

        let p_idx = unsafe { nvim_mtnode_get_p_idx(x) };
        let parent_meta = mtnode_meta(parent, p_idx);
        if meta_has(&parent_meta, meta_filter) {
            return true;
        }

        let n = mtnode_n(x);
        unsafe { nvim_mtitr_set_i(itr, n) };

        // Step to parent
        let _ = marktree_itr_next_skip(b, itr, true, false, std::ptr::null_mut(), std::ptr::null());
    }

    let x = unsafe { nvim_mtitr_get_x(itr) };
    !x.is_null()
}

/// Exported FFI version of `marktree_itr_step_out_filter`.
#[no_mangle]
pub extern "C" fn rs_marktree_itr_step_out_filter(
    b: MarkTreeHandle,
    itr: MarkTreeIterHandle,
    meta_filter: MetaFilter,
) -> bool {
    marktree_itr_step_out_filter(b, itr, meta_filter)
}

// ============================================================================
// Overlap Iteration
// ============================================================================

/// Initialize iterator for overlap queries at a position.
///
/// After calling this, use `marktree_itr_step_overlap` to iterate through
/// all marks that overlap the given position.
#[must_use]
pub fn marktree_itr_get_overlap(
    b: MarkTreeHandle,
    row: i32,
    col: i32,
    itr: MarkTreeIterHandle,
) -> bool {
    if marktree_n_keys(b) == 0 {
        unsafe { nvim_mtitr_set_x(itr, MTNodeHandle::null()) };
        return false;
    }

    let root = marktree_root(b);
    unsafe {
        nvim_mtitr_set_x(itr, root);
        nvim_mtitr_set_i(itr, -1);
        nvim_mtitr_set_lvl(itr, 0);
        nvim_mtitr_set_pos(itr, MTPos::new(0, 0));
        nvim_mtitr_set_intersect_pos(itr, MTPos::new(row, col));
        nvim_mtitr_set_intersect_pos_x(itr, MTPos::new(row, col));
        nvim_mtitr_set_intersect_idx(itr, 0);
    }
    true
}

/// Exported FFI version of `marktree_itr_get_overlap`.
#[no_mangle]
pub extern "C" fn rs_marktree_itr_get_overlap(
    b: MarkTreeHandle,
    row: i32,
    col: i32,
    itr: MarkTreeIterHandle,
) -> bool {
    marktree_itr_get_overlap(b, row, col, itr)
}

/// Step through overlapping mark pairs.
///
/// Returns true if a valid pair was found. When all overlapping pairs
/// have been found, returns false and the iterator becomes a normal
/// iterator at the queried position.
#[allow(clippy::many_single_char_names)] // Matching C code naming conventions
#[allow(clippy::too_many_lines)] // Complex algorithm matching C implementation
#[must_use]
pub fn marktree_itr_step_overlap(
    b: MarkTreeHandle,
    itr: MarkTreeIterHandle,
    pair: &mut MTPair,
) -> bool {
    let mut i = unsafe { nvim_mtitr_get_i(itr) };
    let mut x = unsafe { nvim_mtitr_get_x(itr) };
    let mut lvl = unsafe { nvim_mtitr_get_lvl(itr) };
    let mut pos = unsafe { nvim_mtitr_get_pos(itr) };
    let intersect_pos = unsafe { nvim_mtitr_get_intersect_pos(itr) };
    let mut intersect_pos_x = unsafe { nvim_mtitr_get_intersect_pos_x(itr) };
    let mut intersect_idx = unsafe { nvim_mtitr_get_intersect_idx(itr) };

    // Phase 1: Walk down from root, returning intersections at each ancestor node
    while i == -1 {
        let intersect_size = unsafe { nvim_mtnode_get_intersect_size(x) };
        if intersect_idx < intersect_size {
            let id = unsafe { nvim_mtnode_get_intersect_elem(x, intersect_idx) };
            intersect_idx += 1;
            unsafe { nvim_mtitr_set_intersect_idx(itr, intersect_idx) };

            let start = marktree_lookup(b, id, MarkTreeIterHandle(std::ptr::null_mut()));
            let end = marktree_lookup(
                b,
                id | MARKTREE_END_FLAG,
                MarkTreeIterHandle(std::ptr::null_mut()),
            );
            *pair = MTPair::from_keys(start, end);
            return true;
        }

        if mtnode_level(x) == 0 {
            unsafe {
                nvim_mtitr_set_s_i(itr, lvl, 0);
                nvim_mtitr_set_i(itr, 0);
            }
            i = 0;
            break;
        }

        // Find position in this node
        let k = MTKey {
            pos: intersect_pos_x,
            ns: 0,
            id: 0,
            flags: 0,
            decor_data: DecorInlineData::zero(),
        };
        let (found_i, _) = marktree_getp_aux(x, &k);
        i = found_i + 1;

        unsafe {
            nvim_mtitr_set_s_i(itr, lvl, i);
            nvim_mtitr_set_s_oldcol(itr, lvl, pos.col);
        }

        if i > 0 {
            let key_pos = mtnode_key(x, i - 1).pos;
            compose(&mut pos, key_pos);
            relative(key_pos, &mut intersect_pos_x);
        }

        x = mtnode_ptr(x, i);
        lvl += 1;
        i = -1;
        intersect_idx = 0;

        unsafe {
            nvim_mtitr_set_x(itr, x);
            nvim_mtitr_set_i(itr, i);
            nvim_mtitr_set_lvl(itr, lvl);
            nvim_mtitr_set_pos(itr, pos);
            nvim_mtitr_set_intersect_pos_x(itr, intersect_pos_x);
            nvim_mtitr_set_intersect_idx(itr, intersect_idx);
        }
    }

    // Phase 2: Consider start marks before the queried position
    let n = mtnode_n(x);
    while i < n {
        let key_pos = mtnode_key(x, i).pos;
        if !pos_less(key_pos, intersect_pos_x) {
            break;
        }

        let k = mtnode_key(x, i);
        i += 1;
        unsafe {
            nvim_mtitr_set_i(itr, i);
            nvim_mtitr_set_s_i(itr, lvl, i);
        }

        if mt_start(&k) {
            let end_id = mt_lookup_id(k.ns, k.id, true);
            let end = marktree_lookup(b, end_id, MarkTreeIterHandle(std::ptr::null_mut()));
            if pos_less(end.pos, intersect_pos) {
                continue;
            }

            let mut start = k;
            unrelative(pos, &mut start.pos);
            *pair = MTPair::from_keys(start, end);
            return true;
        }
    }

    // Phase 2B: Consider end marks that might close ranges overlapping the position
    while i < n {
        let k = mtnode_key(x, i);
        i += 1;
        unsafe { nvim_mtitr_set_i(itr, i) };

        if mt_end(&k) {
            let start_id = mt_lookup_id(k.ns, k.id, false);
            let start_node = unsafe { nvim_marktree_id2node(b, start_id) };
            if start_node == x {
                continue;
            }

            let mut end = k;
            unrelative(pos, &mut end.pos);

            let start = marktree_lookup(b, start_id, MarkTreeIterHandle(std::ptr::null_mut()));
            if pos_leq(intersect_pos, start.pos) {
                continue;
            }

            *pair = MTPair::from_keys(start, end);
            return true;
        }
    }

    // Restore iterator to queried position
    let saved_i = unsafe { nvim_mtitr_get_s_i(itr, lvl) };
    unsafe { nvim_mtitr_set_i(itr, saved_i) };

    if saved_i >= n {
        let _ = marktree_itr_next(b, itr);
    }

    false
}

/// Exported FFI version of `marktree_itr_step_overlap`.
#[no_mangle]
pub extern "C" fn rs_marktree_itr_step_overlap(
    b: MarkTreeHandle,
    itr: MarkTreeIterHandle,
    pair: *mut MTPair,
) -> bool {
    unsafe {
        if pair.is_null() {
            return false;
        }
        marktree_itr_step_overlap(b, itr, &mut *pair)
    }
}

// ============================================================================
// Phase 4: Tree Mutation - Insertion
// ============================================================================

/// Branch factor T for the B-tree.
pub const T: usize = MT_BRANCH_FACTOR;

// ============================================================================
// Node Mutation Wrappers
// ============================================================================

/// Set the number of keys in a node.
pub fn mtnode_set_n(x: MTNodeHandle, n: i32) {
    unsafe { nvim_mtnode_set_n(x, n) }
}

/// Set the level of a node.
pub fn mtnode_set_level(x: MTNodeHandle, level: i32) {
    unsafe { nvim_mtnode_set_level(x, level) }
}

/// Set a key in a node at the given index.
pub fn mtnode_set_key(x: MTNodeHandle, idx: i32, k: MTKey) {
    unsafe { nvim_mtnode_set_key(x, idx, k) }
}

/// Set a child pointer in a node at the given index.
pub fn mtnode_set_ptr(x: MTNodeHandle, idx: i32, ptr: MTNodeHandle) {
    unsafe { nvim_mtnode_set_ptr(x, idx, ptr) }
}

/// Set the parent of a node.
pub fn mtnode_set_parent(x: MTNodeHandle, parent: MTNodeHandle) {
    unsafe { nvim_mtnode_set_parent(x, parent) }
}

/// Set the parent index of a node.
pub fn mtnode_set_p_idx(x: MTNodeHandle, p_idx: i32) {
    unsafe { nvim_mtnode_set_p_idx(x, p_idx) }
}

/// Set a meta count for a node child.
pub fn mtnode_set_meta(x: MTNodeHandle, idx: i32, m: i32, val: u32) {
    unsafe { nvim_mtnode_set_meta(x, idx, m, val) }
}

/// Move keys within a node (memmove).
pub fn mtnode_memmove_keys(x: MTNodeHandle, dst: i32, src: i32, count: i32) {
    unsafe { nvim_mtnode_memmove_keys(x, dst, src, count) }
}

/// Move child pointers within a node (memmove).
pub fn mtnode_memmove_ptr(x: MTNodeHandle, dst: i32, src: i32, count: i32) {
    unsafe { nvim_mtnode_memmove_ptr(x, dst, src, count) }
}

/// Move meta arrays within a node (memmove).
pub fn mtnode_memmove_meta(x: MTNodeHandle, dst: i32, src: i32, count: i32) {
    unsafe { nvim_mtnode_memmove_meta(x, dst, src, count) }
}

/// Copy keys from one node to another.
pub fn mtnode_memcpy_keys(
    dst: MTNodeHandle,
    dst_idx: i32,
    src: MTNodeHandle,
    src_idx: i32,
    count: i32,
) {
    unsafe { nvim_mtnode_memcpy_keys(dst, dst_idx, src, src_idx, count) }
}

/// Copy child pointers from one node to another.
pub fn mtnode_memcpy_ptr(
    dst: MTNodeHandle,
    dst_idx: i32,
    src: MTNodeHandle,
    src_idx: i32,
    count: i32,
) {
    unsafe { nvim_mtnode_memcpy_ptr(dst, dst_idx, src, src_idx, count) }
}

/// Copy meta arrays from one node to another.
pub fn mtnode_memcpy_meta(
    dst: MTNodeHandle,
    dst_idx: i32,
    src: MTNodeHandle,
    src_idx: i32,
    count: i32,
) {
    unsafe { nvim_mtnode_memcpy_meta(dst, dst_idx, src, src_idx, count) }
}

// ============================================================================
// Tree Mutation Wrappers
// ============================================================================

/// Allocate a new marktree node.
#[must_use]
pub fn marktree_alloc_node(b: MarkTreeHandle, internal: bool) -> MTNodeHandle {
    unsafe { nvim_marktree_alloc_node(b, internal) }
}

/// Update id2node map for a key at given index.
pub fn marktree_refkey(b: MarkTreeHandle, x: MTNodeHandle, i: i32) {
    unsafe { nvim_marktree_refkey(b, x, i) }
}

/// Set the root node of a marktree.
pub fn marktree_set_root(b: MarkTreeHandle, root: MTNodeHandle) {
    unsafe { nvim_marktree_set_root(b, root) }
}

/// Increment the number of keys in a marktree.
pub fn marktree_inc_n_keys(b: MarkTreeHandle) {
    unsafe { nvim_marktree_inc_n_keys(b) }
}

/// Add to meta_root by index.
pub fn marktree_add_meta_root(b: MarkTreeHandle, m: i32, val: u32) {
    unsafe { nvim_marktree_add_meta_root(b, m, val) }
}

// ============================================================================
// Intersection Wrappers
// ============================================================================

/// Add an intersection to a node (sorted insert).
pub fn intersect_node(_b: MarkTreeHandle, x: MTNodeHandle, id: u64) {
    intersect_node_rs(x, id);
}

/// Remove an intersection from a node.
pub fn unintersect_node(_b: MarkTreeHandle, x: MTNodeHandle, id: u64, strict: bool) {
    unintersect_node_rs(x, id, strict);
}

/// Copy intersections from one node to another.
pub fn kvi_copy_intersect(dst: MTNodeHandle, src: MTNodeHandle) {
    unsafe { nvim_kvi_copy_intersect(dst, src) }
}

/// Check if a node's intersect list contains the given ID.
#[must_use]
pub fn intersection_has(x: MTNodeHandle, id: u64) -> bool {
    let v = read_intersect(x);
    v.binary_search(&id).is_ok()
}

/// Read the intersection list of a node into a Vec.
fn read_intersect(x: MTNodeHandle) -> Vec<u64> {
    let size = unsafe { nvim_mtnode_get_intersect_size(x) };
    let mut v = Vec::with_capacity(size);
    for i in 0..size {
        v.push(unsafe { nvim_mtnode_get_intersect_elem(x, i) });
    }
    v
}

/// Write a Vec back into a node's intersection list (clear then push).
fn write_intersect(x: MTNodeHandle, v: &[u64]) {
    unsafe { nvim_mtnode_intersect_clear(x) };
    for &id in v {
        unsafe { nvim_mtnode_intersect_push(x, id) };
    }
}

// ============================================================================
// Phase 1: Native Rust intersection set operations
// ============================================================================

/// Sorted insert of `id` into node `x`'s intersection list.
///
/// Ported from C `intersect_node`. The list remains sorted after insert.
pub fn intersect_node_rs(x: MTNodeHandle, id: u64) {
    debug_assert!(
        id & MARKTREE_END_FLAG == 0,
        "intersect id must not be end flag"
    );
    let mut v = read_intersect(x);
    // Binary search for insertion point (or existing element)
    match v.binary_search(&id) {
        Ok(_) => {} // already present (shouldn't happen in correct code)
        Err(pos) => v.insert(pos, id),
    }
    write_intersect(x, &v);
}

/// Remove `id` from node `x`'s intersection list.
///
/// Ported from C `unintersect_node`. If `strict` is true, panics if not found
/// (in debug builds only; release builds silently skip).
pub fn unintersect_node_rs(x: MTNodeHandle, id: u64, strict: bool) {
    debug_assert!(
        id & MARKTREE_END_FLAG == 0,
        "unintersect id must not be end flag"
    );
    let mut v = read_intersect(x);
    match v.binary_search(&id) {
        Ok(pos) => {
            v.remove(pos);
        }
        Err(_) => {
            // In C, strict mode has a conditional assert that is disabled in RelWithDebInfo.
            // We mirror the same behavior: panic only in debug builds.
            debug_assert!(!strict, "unintersect_node: id {id} not found in node");
        }
    }
    write_intersect(x, &v);
}

/// Compute set intersection: i = x & y (pure Rust, no node handle needed).
///
/// Ported from C `intersect_common`.
fn intersect_common_vecs(x: &[u64], y: &[u64]) -> Vec<u64> {
    let mut result = Vec::new();
    let mut xi = 0;
    let mut yi = 0;
    while xi < x.len() && yi < y.len() {
        match x[xi].cmp(&y[yi]) {
            std::cmp::Ordering::Equal => {
                result.push(x[xi]);
                xi += 1;
                yi += 1;
            }
            std::cmp::Ordering::Less => xi += 1,
            std::cmp::Ordering::Greater => yi += 1,
        }
    }
    result
}

/// In-place set union: x |= y (pure Rust).
///
/// Ported from C `intersect_add`.
fn intersect_add_vecs(x: &mut Vec<u64>, y: &[u64]) {
    let mut xi = 0;
    let mut yi = 0;
    while xi < x.len() && yi < y.len() {
        match x[xi].cmp(&y[yi]) {
            std::cmp::Ordering::Equal => {
                xi += 1;
                yi += 1;
            }
            std::cmp::Ordering::Greater => {
                x.insert(xi, y[yi]);
                xi += 1; // skip newly inserted element
                yi += 1;
            }
            std::cmp::Ordering::Less => xi += 1,
        }
    }
    // Append remaining y elements
    x.extend_from_slice(&y[yi..]);
}

/// In-place asymmetric difference: x &= ~y (pure Rust).
///
/// Ported from C `intersect_sub`.
fn intersect_sub_vecs(x: &mut Vec<u64>, y: &[u64]) {
    let mut xi = 0;
    let mut yi = 0;
    let mut xn = 0;
    while xi < x.len() && yi < y.len() {
        match x[xi].cmp(&y[yi]) {
            std::cmp::Ordering::Equal => {
                xi += 1;
                yi += 1;
            }
            std::cmp::Ordering::Less => {
                x[xn] = x[xi];
                xn += 1;
                xi += 1;
            }
            std::cmp::Ordering::Greater => yi += 1,
        }
    }
    // Copy remaining x elements
    while xi < x.len() {
        x[xn] = x[xi];
        xn += 1;
        xi += 1;
    }
    x.truncate(xn);
}

/// Extract common elements from x and y, removing them from both.
///
/// After: m = old_x & old_y, x = old_x - m, y = old_y - m.
/// Ported from C `intersect_merge`.
#[allow(dead_code)]
fn intersect_merge_vecs(m: &mut Vec<u64>, x: &mut Vec<u64>, y: &mut Vec<u64>) {
    let mut xi = 0;
    let mut yi = 0;
    let mut xn = 0;
    let mut yn = 0;
    while xi < x.len() && yi < y.len() {
        match x[xi].cmp(&y[yi]) {
            std::cmp::Ordering::Equal => {
                m.push(x[xi]);
                xi += 1;
                yi += 1;
            }
            std::cmp::Ordering::Less => {
                x[xn] = x[xi];
                xn += 1;
                xi += 1;
            }
            std::cmp::Ordering::Greater => {
                y[yn] = y[yi];
                yn += 1;
                yi += 1;
            }
        }
    }
    // Copy remaining elements
    while xi < x.len() {
        x[xn] = x[xi];
        xn += 1;
        xi += 1;
    }
    while yi < y.len() {
        y[yn] = y[yi];
        yn += 1;
        yi += 1;
    }
    x.truncate(xn);
    y.truncate(yn);
}

/// Adjust intersections when child `w` moves from parent `x` to parent `y`.
///
/// `d` receives intersections that must be added to all other children of `y`.
/// Ported from C `intersect_mov`.
///
/// Invariant: x, y, w, d are sorted.
#[allow(dead_code)]
fn intersect_mov_vecs(x: &[u64], y: &mut Vec<u64>, w: &mut Vec<u64>, d: &mut Vec<u64>) {
    let mut wi: usize = 0;
    let mut yi: usize = 0;
    let mut wn: usize = 0;
    let mut yn: usize = 0;
    let mut xi: usize = 0;

    while wi < w.len() || xi < x.len() {
        if wi < w.len() && (xi >= x.len() || x[xi] >= w[wi]) {
            if xi < x.len() && x[xi] == w[wi] {
                xi += 1;
            }
            // now w[wi] not in x strictly (or xi exhausted)
            while yi < y.len() && y[yi] < w[wi] {
                d.push(y[yi]);
                yi += 1;
            }
            if yi < y.len() && y[yi] == w[wi] {
                w[wn] = y[yi]; // keep w[wi] in w (it's also in y)
                yi += 1;
            } else {
                // w[wi] not in y, keep it in w
                w[wn] = w[wi];
            }
            wn += 1;
            wi += 1;
        } else {
            // x[xi] < w[wi] strictly (or wi exhausted)
            while yi < y.len() && y[yi] < x[xi] {
                d.push(y[yi]);
                yi += 1;
            }
            if yi < y.len() && y[yi] == x[xi] {
                y[yn] = y[yi]; // keep in y (it's also in x)
                yn += 1;
                yi += 1;
            } else {
                // x[xi] not in y: add x[xi] to w at position wn
                if wi == wn {
                    w.insert(wn, x[xi]);
                    wn += 1;
                    wi += 1; // skip newly added element
                } else {
                    // wn < wi: just store at wn without shifting
                    w[wn] = x[xi];
                    wn += 1;
                }
            }
            xi += 1;
        }
    }
    // Move remaining y elements to d
    while yi < y.len() {
        d.push(y[yi]);
        yi += 1;
    }
    w.truncate(wn);
    y.truncate(yn);
}

/// Bubble up intersections common to all children of `x` into `x` itself.
///
/// `x` is a node which shrunk or is the half of a split.
/// Ported from C `bubble_up`.
pub fn bubble_up_rs(x: MTNodeHandle) {
    let n = mtnode_n(x);
    // The largest common subset is the intersection of first and last child
    let first_child = mtnode_ptr(x, 0);
    let last_child = mtnode_ptr(x, n);
    let first_v = read_intersect(first_child);
    let last_v = read_intersect(last_child);
    let common = intersect_common_vecs(&first_v, &last_v);
    if common.is_empty() {
        return;
    }
    // Remove common from all children
    for i in 0..=n {
        let child = mtnode_ptr(x, i);
        let mut child_v = read_intersect(child);
        intersect_sub_vecs(&mut child_v, &common);
        write_intersect(child, &child_v);
    }
    // Add common to x itself
    let mut x_v = read_intersect(x);
    intersect_add_vecs(&mut x_v, &common);
    write_intersect(x, &x_v);
}

// ============================================================================
// B-tree Insertion (Phase 3)
// ============================================================================

/// Split a full child node during insertion.
///
/// x must be an internal node, which is not full.
/// x->ptr[i] should be a full node, i.e. x->ptr[i]->n == 2*T-1.
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::needless_range_loop,
    clippy::many_single_char_names
)]
pub fn split_node(b: MarkTreeHandle, x: MTNodeHandle, i: i32, next: MTKey) {
    let y = unsafe { nvim_mtnode_get_ptr(x, i) };
    let y_level = unsafe { nvim_mtnode_get_level(y) };
    let z = marktree_alloc_node(b, y_level != 0);
    unsafe {
        nvim_mtnode_set_level(z, y_level);
        nvim_mtnode_set_n(z, (T as i32) - 1);
    }

    // tricky: we might split a node in between inserting the start node and the end
    // node of the same pair. Then we must not intersect this id yet (done later
    // in marktree_intersect_pair).
    let last_start = if mt_end(&next) {
        mt_lookup_id(next.ns, next.id, false)
    } else {
        MARKTREE_END_FLAG
    };

    // no alloc in the common case (less than 4 intersects)
    unsafe { nvim_kvi_copy_intersect(z, y) };

    if y_level == 0 {
        let pi = pseudo_index(y, 0); // note: sloppy pseudo-index
        for j in 0..(T as i32) {
            let k = unsafe { nvim_mtnode_get_key(y, j) };
            let pi_end = pseudo_index_for_id(b, mt_lookup_id(k.ns, k.id, true), true);
            if mt_start(&k) && pi_end > pi && mt_lookup_key(&k) != last_start {
                intersect_node(b, z, mt_lookup_id(k.ns, k.id, false));
            }
        }

        // note: y->key[T-1] is moved up and thus checked for both
        for j in ((T as i32) - 1)..((T as i32) * 2 - 1) {
            let k = unsafe { nvim_mtnode_get_key(y, j) };
            let pi_start = pseudo_index_for_id(b, mt_lookup_id(k.ns, k.id, false), true);
            if mt_end(&k) && pi_start > 0 && pi_start < pi {
                intersect_node(b, y, mt_lookup_id(k.ns, k.id, false));
            }
        }
    }

    // Copy upper half of y's keys into z
    unsafe { nvim_mtnode_memcpy_keys(z, 0, y, T as i32, (T as i32) - 1) };
    for j in 0..((T as i32) - 1) {
        marktree_refkey(b, z, j);
    }

    if y_level != 0 {
        // Copy upper half of y's ptr and meta into z
        unsafe { nvim_mtnode_memcpy_ptr(z, 0, y, T as i32, T as i32) };
        unsafe { nvim_mtnode_memcpy_meta(z, 0, y, T as i32, T as i32) };
        for j in 0..(T as i32) {
            let child = unsafe { nvim_mtnode_get_ptr(z, j) };
            unsafe {
                nvim_mtnode_set_parent(child, z);
                nvim_mtnode_set_p_idx(child, j);
            }
        }
    }

    unsafe { nvim_mtnode_set_n(y, (T as i32) - 1) };

    let x_n = unsafe { nvim_mtnode_get_n(x) };
    // Make room in x for z (shift ptr and meta right)
    unsafe { nvim_mtnode_memmove_ptr(x, i + 2, i + 1, x_n - i) };
    unsafe { nvim_mtnode_memmove_meta(x, i + 2, i + 1, x_n - i) };
    unsafe { nvim_mtnode_set_ptr(x, i + 1, z) };

    // Compute and store meta for z
    let z_meta = meta_describe_node(z);
    for m in 0..K_MT_META_COUNT {
        unsafe { nvim_mtnode_set_meta(x, i + 1, m as i32, z_meta[m]) };
    }

    unsafe { nvim_mtnode_set_parent(z, x) }; // == y->parent

    // Fix p_idx for all children from i+1 to x->n+1
    for j in (i + 1)..=(x_n + 1) {
        let child = unsafe { nvim_mtnode_get_ptr(x, j) };
        unsafe { nvim_mtnode_set_p_idx(child, j) };
    }

    // Shift keys right to make room for promoted key
    unsafe { nvim_mtnode_memmove_keys(x, i + 1, i, x_n - i) };

    // Move key to internal layer: x->key[i] = y->key[T-1]
    let promoted = unsafe { nvim_mtnode_get_key(y, (T as i32) - 1) };
    unsafe { nvim_mtnode_set_key(x, i, promoted) };
    marktree_refkey(b, x, i);
    unsafe { nvim_mtnode_set_n(x, x_n + 1) };

    let x_key_i = unsafe { nvim_mtnode_get_key(x, i) };
    let meta_inc = meta_describe_key(&x_key_i);
    // y used to contain all of z and x->key[i]; discount those from y's meta
    for m in 0..K_MT_META_COUNT {
        let old_y_meta = unsafe { nvim_mtnode_get_meta(x, i, m as i32) };
        let z_m = unsafe { nvim_mtnode_get_meta(x, i + 1, m as i32) };
        unsafe {
            nvim_mtnode_set_meta(x, i, m as i32, old_y_meta - z_m - meta_inc[m]);
        }
    }

    // Adjust relative positions for z's keys (make them relative to promoted key)
    for j in 0..((T as i32) - 1) {
        let mut zk = unsafe { nvim_mtnode_get_key(z, j) };
        relative(x_key_i.pos, &mut zk.pos);
        unsafe { nvim_mtnode_set_key(z, j, zk) };
    }

    // Adjust promoted key position (make relative to previous key in x)
    if i > 0 {
        let prev_key = unsafe { nvim_mtnode_get_key(x, i - 1) };
        let mut promoted_key = unsafe { nvim_mtnode_get_key(x, i) };
        unrelative(prev_key.pos, &mut promoted_key.pos);
        unsafe { nvim_mtnode_set_key(x, i, promoted_key) };
    }

    if y_level != 0 {
        bubble_up(y);
        bubble_up(z);
    }
}

/// Recursive insertion helper.
///
/// x must not be a full node (even if there might be internal space).
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::needless_range_loop,
    clippy::many_single_char_names
)]
pub fn marktree_putp_aux(
    b: MarkTreeHandle,
    x: MTNodeHandle,
    mut k: MTKey,
    meta_inc: &mut [u32; K_MT_META_COUNT],
) {
    // TODO(bfredl): ugh, make sure this is the _last_ valid (pos, gravity) position,
    // to minimize movement
    let (getp_i, _) = marktree_getp_aux(x, &k);
    let i = getp_i + 1;
    let x_level = unsafe { nvim_mtnode_get_level(x) };
    if x_level == 0 {
        let x_n = unsafe { nvim_mtnode_get_n(x) };
        if i != x_n {
            unsafe { nvim_mtnode_memmove_keys(x, i + 1, i, x_n - i) };
        }
        unsafe { nvim_mtnode_set_key(x, i, k) };
        marktree_refkey(b, x, i);
        unsafe { nvim_mtnode_set_n(x, x_n + 1) };
    } else {
        let child_i = unsafe { nvim_mtnode_get_ptr(x, i) };
        let child_n = unsafe { nvim_mtnode_get_n(child_i) };
        let actual_i = if child_n == (2 * T as i32) - 1 {
            split_node(b, x, i, k);
            if key_cmp(&k, &unsafe { nvim_mtnode_get_key(x, i) }) > 0 {
                i + 1
            } else {
                i
            }
        } else {
            i
        };
        if actual_i > 0 {
            let prev_key = unsafe { nvim_mtnode_get_key(x, actual_i - 1) };
            relative(prev_key.pos, &mut k.pos);
        }
        marktree_putp_aux(b, unsafe { nvim_mtnode_get_ptr(x, actual_i) }, k, meta_inc);
        for m in 0..K_MT_META_COUNT {
            let old = unsafe { nvim_mtnode_get_meta(x, actual_i, m as i32) };
            unsafe { nvim_mtnode_set_meta(x, actual_i, m as i32, old + meta_inc[m]) };
        }
    }
}

/// Insert a key into the marktree.
///
/// This is the core insertion function. It handles root splitting
/// and delegates to putp_aux for the actual insertion.
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::needless_range_loop,
    clippy::many_single_char_names
)]
pub fn marktree_put_key(b: MarkTreeHandle, mut k: MTKey) {
    k.flags |= MT_FLAG_REAL; // let's be real.
    let mut r = unsafe { nvim_marktree_get_root(b) };
    if r.is_null() {
        r = marktree_alloc_node(b, true);
        marktree_set_root(b, r);
    }

    let r_n = unsafe { nvim_mtnode_get_n(r) };
    if r_n == (2 * T as i32) - 1 {
        let s = marktree_alloc_node(b, true);
        marktree_set_root(b, s);
        let r_level = unsafe { nvim_mtnode_get_level(r) };
        unsafe { nvim_mtnode_set_level(s, r_level + 1) };
        unsafe { nvim_mtnode_set_n(s, 0) };
        unsafe { nvim_mtnode_set_ptr(s, 0, r) };
        // s->meta[0] = b->meta_root
        let meta_root = marktree_meta_root(b);
        for m in 0..K_MT_META_COUNT {
            unsafe { nvim_mtnode_set_meta(s, 0, m as i32, meta_root[m]) };
        }
        unsafe { nvim_mtnode_set_parent(r, s) };
        unsafe { nvim_mtnode_set_p_idx(r, 0) };
        split_node(b, s, 0, k);
        r = s;
    }

    let mut meta_inc = meta_describe_key(&k);
    marktree_putp_aux(b, r, k, &mut meta_inc);
    for m in 0..K_MT_META_COUNT {
        unsafe { nvim_marktree_add_meta_root(b, m as i32, meta_inc[m]) };
    }
    marktree_inc_n_keys(b);
}

/// Insert a mark with optional paired end.
///
/// If end_row >= 0, creates a paired mark with the end at (end_row, end_col).
/// The end mark will have right gravity if end_right is true.
///
/// # Panics
///
/// Panics if key.flags contains invalid external bits.
#[allow(clippy::many_single_char_names)]
pub fn marktree_put(
    b: MarkTreeHandle,
    mut key: MTKey,
    end_row: i32,
    end_col: i32,
    end_right: bool,
) {
    assert!(key.flags & !(MT_FLAG_EXTERNAL_MASK | MT_FLAG_RIGHT_GRAVITY) == 0);
    if end_row >= 0 {
        key.flags |= MT_FLAG_PAIRED;
    }

    marktree_put_key(b, key);

    if end_row >= 0 {
        let mut end_key = key;
        end_key.flags = (key.flags & !MT_FLAG_RIGHT_GRAVITY)
            | MT_FLAG_END
            | if end_right { MT_FLAG_RIGHT_GRAVITY } else { 0 };
        end_key.pos = MTPos {
            row: end_row,
            col: end_col,
        };
        marktree_put_key(b, end_key);
        let itr = unsafe { nvim_alloc_marktreeiter() };
        let end_itr = unsafe { nvim_alloc_marktreeiter() };
        let _ = marktree_lookup(b, mt_lookup_key(&key), itr);
        let _ = marktree_lookup(b, mt_lookup_key(&end_key), end_itr);
        marktree_intersect_pair(b, mt_lookup_key(&key), itr, end_itr, false);
        unsafe {
            nvim_free_marktreeiter(itr);
            nvim_free_marktreeiter(end_itr);
        }
    }
}

/// Mark intersections between paired marks.
///
/// Traverses from itr to end_itr, adding (or removing if delete=true)
/// intersection markers for the paired mark identified by id.
/// Mark intersections between paired marks.
///
/// `itr` is the iterator for the start mark (mutated during traversal).
/// `end_itr` is the iterator for the end mark (not mutated).
/// For every internal-node child between the two marks, adds (or removes
/// if `delete`) `id` from the child's intersection list.
/// Ported from C `marktree_intersect_pair`.
pub fn marktree_intersect_pair(
    b: MarkTreeHandle,
    id: u64,
    itr: MarkTreeIterHandle,
    end_itr: MarkTreeIterHandle,
    delete: bool,
) {
    // iat(it, l, q) = if l == it->lvl { it->i + q } else { it->s[l].i }
    let iat = |it: MarkTreeIterHandle, l: i32, q: i32| -> i32 {
        let lvl = unsafe { nvim_mtitr_get_lvl(it) };
        if l == lvl {
            (unsafe { nvim_mtitr_get_i(it) }) + q
        } else {
            unsafe { nvim_mtitr_get_s_i(it, l) }
        }
    };

    let itr_lvl_init = unsafe { nvim_mtitr_get_lvl(itr) };
    let end_lvl = unsafe { nvim_mtitr_get_lvl(end_itr) };
    let maxlvl = itr_lvl_init.min(end_lvl);

    let mut lvl = 0i32;
    while lvl < maxlvl {
        let si = unsafe { nvim_mtitr_get_s_i(itr, lvl) };
        let esi = unsafe { nvim_mtitr_get_s_i(end_itr, lvl) };
        if si > esi {
            return; // empty range
        } else if si < esi {
            break; // work to do
        }
        lvl += 1;
    }
    if lvl == maxlvl && iat(itr, lvl, 1) > iat(end_itr, lvl, 0) {
        return; // empty range
    }

    loop {
        let itr_x = unsafe { nvim_mtitr_get_x(itr) };
        if itr_x.is_null() {
            break;
        }
        let end_x = unsafe { nvim_mtitr_get_x(end_itr) };
        let itr_i = unsafe { nvim_mtitr_get_i(itr) };
        let end_i = unsafe { nvim_mtitr_get_i(end_itr) };
        let itr_level = unsafe { nvim_mtnode_get_level(itr_x) };
        let itr_cur_lvl = unsafe { nvim_mtitr_get_lvl(itr) };

        let skip = if itr_x == end_x {
            if itr_level == 0 || itr_i >= end_i {
                break;
            }
            true
        } else if itr_cur_lvl > lvl || iat(itr, lvl, 1) < iat(end_itr, lvl, 1) {
            true
        } else {
            lvl += 1;
            false
        };

        if skip && itr_level != 0 {
            let child = unsafe { nvim_mtnode_get_ptr(itr_x, itr_i + 1) };
            if delete {
                unintersect_node(b, child, id, true);
            } else {
                intersect_node(b, child, id);
            }
        }
        let _ = marktree_itr_next_skip(
            b,
            itr,
            skip,
            true,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
    }
}

/// Bubble up common intersections to parent.
pub fn bubble_up(x: MTNodeHandle) {
    bubble_up_rs(x);
}

// ============================================================================
// FFI Exports for Phase 4
// ============================================================================

/// Exported FFI version of `marktree_alloc_node`.
#[no_mangle]
pub extern "C" fn rs_marktree_alloc_node(b: MarkTreeHandle, internal: bool) -> MTNodeHandle {
    marktree_alloc_node(b, internal)
}

/// Exported FFI version of `marktree_refkey`.
#[no_mangle]
pub extern "C" fn rs_marktree_refkey(b: MarkTreeHandle, x: MTNodeHandle, i: c_int) {
    marktree_refkey(b, x, i);
}

/// Exported FFI version of `split_node`.
#[no_mangle]
pub extern "C" fn rs_split_node(b: MarkTreeHandle, x: MTNodeHandle, i: c_int, next: MTKey) {
    split_node(b, x, i, next);
}

/// Exported FFI version of `marktree_putp_aux`.
#[no_mangle]
pub extern "C" fn rs_marktree_putp_aux(
    b: MarkTreeHandle,
    x: MTNodeHandle,
    k: MTKey,
    meta_inc: *mut u32,
) {
    unsafe {
        if !meta_inc.is_null() {
            let mut meta = [0u32; K_MT_META_COUNT];
            for (i, m) in meta.iter_mut().enumerate() {
                *m = *meta_inc.add(i);
            }
            marktree_putp_aux(b, x, k, &mut meta);
            for (i, m) in meta.iter().enumerate() {
                *meta_inc.add(i) = *m;
            }
        }
    }
}

/// Exported FFI version of `marktree_put_key`.
#[no_mangle]
pub extern "C" fn rs_marktree_put_key(b: MarkTreeHandle, k: MTKey) {
    marktree_put_key(b, k);
}

/// Exported FFI version of `marktree_put`.
#[no_mangle]
pub extern "C" fn rs_marktree_put(
    b: MarkTreeHandle,
    key: MTKey,
    end_row: c_int,
    end_col: c_int,
    end_right: bool,
) {
    marktree_put(b, key, end_row, end_col, end_right);
}

/// Exported FFI version of `marktree_intersect_pair`.
#[no_mangle]
pub extern "C" fn rs_marktree_intersect_pair(
    b: MarkTreeHandle,
    id: u64,
    itr: MarkTreeIterHandle,
    end_itr: MarkTreeIterHandle,
    delete: bool,
) {
    marktree_intersect_pair(b, id, itr, end_itr, delete);
}

/// Exported FFI version of `intersect_node`.
#[no_mangle]
pub extern "C" fn rs_intersect_node(b: MarkTreeHandle, x: MTNodeHandle, id: u64) {
    intersect_node(b, x, id);
}

/// Exported FFI version of `unintersect_node`.
#[no_mangle]
pub extern "C" fn rs_unintersect_node(b: MarkTreeHandle, x: MTNodeHandle, id: u64, strict: bool) {
    unintersect_node(b, x, id, strict);
}

/// Exported FFI version of `bubble_up`.
#[no_mangle]
pub extern "C" fn rs_bubble_up(x: MTNodeHandle) {
    bubble_up(x);
}

// ============================================================================
// Phase 5: Tree Mutation - Deletion
// ============================================================================

// ============================================================================
// Rebalancing helpers: merge_node, pivot_right, pivot_left
// ============================================================================

/// Merge two sibling nodes during B-tree rebalancing.
///
/// Merges `p->ptr[i]` (left, x) with `p->ptr[i+1]` (right, y).
/// The separator key `p->key[i]` is moved into x, then all keys/children
/// from y are appended. Intersection lists merged via `rs_merge_node_intersect`
/// for internal nodes. The right node y is freed. Returns the merged left node.
#[must_use]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::many_single_char_names
)]
pub fn merge_node(b: MarkTreeHandle, p: MTNodeHandle, i: i32) -> MTNodeHandle {
    let x = unsafe { nvim_mtnode_get_ptr(p, i) };
    let y = unsafe { nvim_mtnode_get_ptr(p, i + 1) };
    let x_old_n = unsafe { nvim_mtnode_get_n(x) }; // save before modification

    // Move separator key p->key[i] into x->key[x_old_n]
    let sep_key = unsafe { nvim_mtnode_get_key(p, i) };
    unsafe { nvim_mtnode_set_key(x, x_old_n, sep_key) };
    marktree_refkey(b, x, x_old_n);
    if i > 0 {
        let prev_pos = unsafe { nvim_mtnode_get_key(p, i - 1) }.pos;
        let mut sep_adjusted = unsafe { nvim_mtnode_get_key(x, x_old_n) };
        relative(prev_pos, &mut sep_adjusted.pos);
        unsafe { nvim_mtnode_set_key(x, x_old_n, sep_adjusted) };
    }

    // Compute meta for the separator key (possibly relativized)
    let sep_key_adjusted = unsafe { nvim_mtnode_get_key(x, x_old_n) };
    let meta_inc = meta_describe_key(&sep_key_adjusted);

    // Copy y's keys into x at positions x_old_n+1..x_old_n+1+y_n
    let y_n = unsafe { nvim_mtnode_get_n(y) };
    unsafe { nvim_mtnode_memcpy_keys(x, x_old_n + 1, y, 0, y_n) };
    for k in 0..y_n {
        marktree_refkey(b, x, x_old_n + 1 + k);
        let mut moved_key = unsafe { nvim_mtnode_get_key(x, x_old_n + 1 + k) };
        unrelative(sep_key_adjusted.pos, &mut moved_key.pos);
        unsafe { nvim_mtnode_set_key(x, x_old_n + 1 + k, moved_key) };
    }

    let x_level = unsafe { nvim_mtnode_get_level(x) };
    if x_level != 0 {
        // Copy y's children/meta into x
        unsafe {
            nvim_mtnode_memcpy_ptr(x, x_old_n + 1, y, 0, y_n + 1);
            nvim_mtnode_memcpy_meta(x, x_old_n + 1, y, 0, y_n + 1);
        }
        // Fix parent/p_idx for y's children now in x
        for ky in 0..=y_n {
            let k = x_old_n + ky + 1;
            let child = unsafe { nvim_mtnode_get_ptr(x, k) };
            unsafe {
                nvim_mtnode_set_parent(child, x);
                nvim_mtnode_set_p_idx(child, k);
            }
        }
        // Merge intersection lists
        crate::intersect_ops::rs_merge_node_intersect(b, x, x_old_n, y, y_n);
    }

    // x->n += y->n + 1
    let new_x_n = x_old_n + y_n + 1;
    unsafe { nvim_mtnode_set_n(x, new_x_n) };

    // Update parent meta: p->meta[i] += p->meta[i+1] + meta_inc
    for m in 0..K_MT_META_COUNT as i32 {
        let m_i = unsafe { nvim_mtnode_get_meta(p, i, m) };
        let m_i1 = unsafe { nvim_mtnode_get_meta(p, i + 1, m) };
        unsafe {
            nvim_mtnode_set_meta(p, i, m, m_i + m_i1 + meta_inc[m as usize]);
        }
    }

    // Remove separator key from parent: memmove p->key[i] <- p->key[i+1]
    let p_n = unsafe { nvim_mtnode_get_n(p) };
    let count = p_n - i - 1;
    if count > 0 {
        unsafe { nvim_mtnode_memmove_keys(p, i, i + 1, count) };
        unsafe { nvim_mtnode_memmove_ptr(p, i + 1, i + 2, count) };
        unsafe { nvim_mtnode_memmove_meta(p, i + 1, i + 2, count) };
    }
    // Fix p_idx for shifted children
    for j in (i + 1)..p_n {
        let child = unsafe { nvim_mtnode_get_ptr(p, j) };
        unsafe { nvim_mtnode_set_p_idx(child, j) };
    }
    unsafe { nvim_mtnode_set_n(p, p_n - 1) };

    // Free the right node y
    marktree_free_node(b, y);

    x
}

/// Pivot right: steal last key from left sibling `x = p->ptr[i]`.
///
/// Moves `x->key[x->n - 1]` up to `p->key[i]` (new separator), pushes the
/// old separator down to `y->key[0]` (right sibling `p->ptr[i+1]`).
/// Adjusts positions, meta counts, and intersection lists.
/// `p_pos` is the absolute position of the parent context (unused inside).
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::many_single_char_names
)]
pub fn pivot_right(b: MarkTreeHandle, _p_pos: MTPos, p: MTNodeHandle, i: i32) {
    let x = unsafe { nvim_mtnode_get_ptr(p, i) };
    let y = unsafe { nvim_mtnode_get_ptr(p, i + 1) };
    let y_n = unsafe { nvim_mtnode_get_n(y) };
    let y_level = unsafe { nvim_mtnode_get_level(y) };

    // Shift y's keys right by 1
    if y_n > 0 {
        unsafe { nvim_mtnode_memmove_keys(y, 1, 0, y_n) };
    }
    if y_level != 0 {
        unsafe { nvim_mtnode_memmove_ptr(y, 1, 0, y_n + 1) };
        unsafe { nvim_mtnode_memmove_meta(y, 1, 0, y_n + 1) };
        for j in 1..=y_n + 1 {
            let child = unsafe { nvim_mtnode_get_ptr(y, j) };
            unsafe { nvim_mtnode_set_p_idx(child, j) };
        }
    }

    // y->key[0] = p->key[i] (separator goes down to y)
    let sep_key = unsafe { nvim_mtnode_get_key(p, i) };
    unsafe { nvim_mtnode_set_key(y, 0, sep_key) };
    marktree_refkey(b, y, 0);

    // p->key[i] = x->key[x->n - 1] (last key of x becomes new separator)
    let x_n = unsafe { nvim_mtnode_get_n(x) };
    let stolen_key = unsafe { nvim_mtnode_get_key(x, x_n - 1) };
    unsafe { nvim_mtnode_set_key(p, i, stolen_key) };
    marktree_refkey(b, p, i);

    let meta_inc_y = meta_describe_key(&unsafe { nvim_mtnode_get_key(y, 0) });
    let meta_inc_x = meta_describe_key(&unsafe { nvim_mtnode_get_key(p, i) });
    for m in 0..K_MT_META_COUNT as i32 {
        let m_i1 = unsafe { nvim_mtnode_get_meta(p, i + 1, m) };
        unsafe { nvim_mtnode_set_meta(p, i + 1, m, m_i1 + meta_inc_y[m as usize]) };
        let m_i = unsafe { nvim_mtnode_get_meta(p, i, m) };
        unsafe { nvim_mtnode_set_meta(p, i, m, m_i - meta_inc_x[m as usize]) };
    }

    let x_level = unsafe { nvim_mtnode_get_level(x) };
    if x_level != 0 {
        // y->ptr[0] = x->ptr[x->n]
        let stolen_child = unsafe { nvim_mtnode_get_ptr(x, x_n) };
        unsafe { nvim_mtnode_set_ptr(y, 0, stolen_child) };
        unsafe { nvim_mtnode_memcpy_meta(y, 0, x, x_n, 1) };
        for m in 0..K_MT_META_COUNT as i32 {
            let child_meta = unsafe { nvim_mtnode_get_meta(y, 0, m) };
            let m_i1 = unsafe { nvim_mtnode_get_meta(p, i + 1, m) };
            unsafe { nvim_mtnode_set_meta(p, i + 1, m, m_i1 + child_meta) };
            let m_i = unsafe { nvim_mtnode_get_meta(p, i, m) };
            unsafe { nvim_mtnode_set_meta(p, i, m, m_i - child_meta) };
        }
        unsafe {
            nvim_mtnode_set_parent(stolen_child, y);
            nvim_mtnode_set_p_idx(stolen_child, 0);
        }
    }

    unsafe { nvim_mtnode_set_n(x, x_n - 1) };
    unsafe { nvim_mtnode_set_n(y, y_n + 1) };

    // Position adjustment
    if i > 0 {
        let prev_pos = unsafe { nvim_mtnode_get_key(p, i - 1) }.pos;
        let mut sep = unsafe { nvim_mtnode_get_key(p, i) };
        unrelative(prev_pos, &mut sep.pos);
        unsafe { nvim_mtnode_set_key(p, i, sep) };
    }
    let new_sep_pos = unsafe { nvim_mtnode_get_key(p, i) }.pos;
    let mut y0 = unsafe { nvim_mtnode_get_key(y, 0) };
    relative(new_sep_pos, &mut y0.pos);
    unsafe { nvim_mtnode_set_key(y, 0, y0) };
    let new_y0_pos = unsafe { nvim_mtnode_get_key(y, 0) }.pos;
    let new_y_n = unsafe { nvim_mtnode_get_n(y) };
    for k in 1..new_y_n {
        let mut yk = unsafe { nvim_mtnode_get_key(y, k) };
        unrelative(new_y0_pos, &mut yk.pos);
        unsafe { nvim_mtnode_set_key(y, k, yk) };
    }

    // Intersection repair
    if x_level != 0 {
        crate::intersect_ops::rs_pivot_right_intersect(b, x, y, new_y_n);
        bubble_up(x);
    } else {
        let new_sep = unsafe { nvim_mtnode_get_key(p, i) };
        if mt_end(&new_sep) {
            let pi = pseudo_index(x, 0);
            let start_id = mt_lookup_key_side(&new_sep, false);
            let pi_start = pseudo_index_for_id(b, start_id, true);
            if pi_start > 0 && pi_start < pi {
                intersect_node(b, x, start_id);
            }
        }
        let new_y0 = unsafe { nvim_mtnode_get_key(y, 0) };
        if mt_start(&new_y0) {
            unintersect_node(b, y, mt_lookup_key(&new_y0), false);
        }
    }
}

/// Pivot left: steal first key from right sibling `y = p->ptr[i+1]`.
///
/// Moves `y->key[0]` up to `p->key[i]` (new separator), pushes the
/// old separator down to `x->key[x->n]` (left sibling `p->ptr[i]`).
/// Adjusts positions, meta counts, and intersection lists.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::many_single_char_names
)]
pub fn pivot_left(b: MarkTreeHandle, _p_pos: MTPos, p: MTNodeHandle, i: i32) {
    let x = unsafe { nvim_mtnode_get_ptr(p, i) };
    let y = unsafe { nvim_mtnode_get_ptr(p, i + 1) };
    let y_n = unsafe { nvim_mtnode_get_n(y) };
    let y_level = unsafe { nvim_mtnode_get_level(y) };

    // Reverse relative encoding in y: for k in 1..y->n: relative(y->key[0], &y->key[k])
    let y0_pos = unsafe { nvim_mtnode_get_key(y, 0) }.pos;
    for k in 1..y_n {
        let mut yk = unsafe { nvim_mtnode_get_key(y, k) };
        relative(y0_pos, &mut yk.pos);
        unsafe { nvim_mtnode_set_key(y, k, yk) };
    }
    // rs_unrelative(p->key[i].pos, &y->key[0].pos)
    let sep_pos = unsafe { nvim_mtnode_get_key(p, i) }.pos;
    let mut y0 = unsafe { nvim_mtnode_get_key(y, 0) };
    unrelative(sep_pos, &mut y0.pos);
    unsafe { nvim_mtnode_set_key(y, 0, y0) };
    // if i > 0: rs_relative(p->key[i-1].pos, &p->key[i].pos)
    if i > 0 {
        let prev_pos = unsafe { nvim_mtnode_get_key(p, i - 1) }.pos;
        let mut sep = unsafe { nvim_mtnode_get_key(p, i) };
        relative(prev_pos, &mut sep.pos);
        unsafe { nvim_mtnode_set_key(p, i, sep) };
    }

    // x->key[x->n] = p->key[i]; p->key[i] = y->key[0]
    let x_n = unsafe { nvim_mtnode_get_n(x) };
    let sep_key = unsafe { nvim_mtnode_get_key(p, i) };
    unsafe { nvim_mtnode_set_key(x, x_n, sep_key) };
    marktree_refkey(b, x, x_n);
    let stolen_key = unsafe { nvim_mtnode_get_key(y, 0) };
    unsafe { nvim_mtnode_set_key(p, i, stolen_key) };
    marktree_refkey(b, p, i);

    let meta_inc_x = meta_describe_key(&unsafe { nvim_mtnode_get_key(x, x_n) });
    let meta_inc_y = meta_describe_key(&unsafe { nvim_mtnode_get_key(p, i) });
    for m in 0..K_MT_META_COUNT as i32 {
        let m_i = unsafe { nvim_mtnode_get_meta(p, i, m) };
        unsafe { nvim_mtnode_set_meta(p, i, m, m_i + meta_inc_x[m as usize]) };
        let m_i1 = unsafe { nvim_mtnode_get_meta(p, i + 1, m) };
        unsafe { nvim_mtnode_set_meta(p, i + 1, m, m_i1 - meta_inc_y[m as usize]) };
    }

    let x_level = unsafe { nvim_mtnode_get_level(x) };
    if x_level != 0 {
        // x->ptr[x->n+1] = y->ptr[0]; copy meta
        let stolen_child = unsafe { nvim_mtnode_get_ptr(y, 0) };
        unsafe { nvim_mtnode_set_ptr(x, x_n + 1, stolen_child) };
        unsafe { nvim_mtnode_memcpy_meta(x, x_n + 1, y, 0, 1) };
        for m in 0..K_MT_META_COUNT as i32 {
            let child_meta = unsafe { nvim_mtnode_get_meta(y, 0, m) };
            let m_i1 = unsafe { nvim_mtnode_get_meta(p, i + 1, m) };
            unsafe { nvim_mtnode_set_meta(p, i + 1, m, m_i1 - child_meta) };
            let m_i = unsafe { nvim_mtnode_get_meta(p, i, m) };
            unsafe { nvim_mtnode_set_meta(p, i, m, m_i + child_meta) };
        }
        unsafe {
            nvim_mtnode_set_parent(stolen_child, x);
            nvim_mtnode_set_p_idx(stolen_child, x_n + 1);
        }
    }

    // Shift y's keys left: memmove(y->key, &y->key[1], y->n - 1)
    if y_n - 1 > 0 {
        unsafe { nvim_mtnode_memmove_keys(y, 0, 1, y_n - 1) };
    }
    if y_level != 0 {
        unsafe { nvim_mtnode_memmove_ptr(y, 0, 1, y_n) };
        unsafe { nvim_mtnode_memmove_meta(y, 0, 1, y_n) };
        for j in 0..y_n {
            let child = unsafe { nvim_mtnode_get_ptr(y, j) };
            unsafe { nvim_mtnode_set_p_idx(child, j) };
        }
    }

    unsafe { nvim_mtnode_set_n(x, x_n + 1) };
    unsafe { nvim_mtnode_set_n(y, y_n - 1) };

    // Intersection repair
    if x_level != 0 {
        let new_x_n = unsafe { nvim_mtnode_get_n(x) };
        crate::intersect_ops::rs_pivot_left_intersect(b, x, new_x_n, y);
        bubble_up(y);
    } else {
        let new_sep = unsafe { nvim_mtnode_get_key(p, i) };
        if mt_start(&new_sep) {
            let pi = pseudo_index(y, 0);
            let end_id = mt_lookup_key_side(&new_sep, true);
            let pi_end = pseudo_index_for_id(b, end_id, true);
            if pi_end > pi {
                intersect_node(b, y, mt_lookup_key(&new_sep));
            }
        }
        let new_x_n = unsafe { nvim_mtnode_get_n(x) };
        let last_x_key = unsafe { nvim_mtnode_get_key(x, new_x_n - 1) };
        if mt_end(&last_x_key) {
            unintersect_node(b, x, mt_lookup_key_side(&last_x_key, false), false);
        }
    }
}

// ============================================================================
// Deletion Wrappers
// ============================================================================

/// Tracks which index field `lasti` points to during the rebalancing loop.
///
/// Mirrors C's `int *lasti` which alternates between `&itr->i` and
/// `&itr->s[rlvl].i` depending on whether we've already moved up one level.
enum LastiRef {
    /// `lasti == &itr->i`
    ItrI,
    /// `lasti == &itr->s[level].i`
    S(i32),
}

impl LastiRef {
    fn get(&self, itr: MarkTreeIterHandle) -> i32 {
        match self {
            Self::ItrI => unsafe { nvim_mtitr_get_i(itr) },
            Self::S(lvl) => unsafe { nvim_mtitr_get_s_i(itr, *lvl) },
        }
    }

    fn set(&self, itr: MarkTreeIterHandle, val: i32) {
        match self {
            Self::ItrI => unsafe { nvim_mtitr_set_i(itr, val) },
            Self::S(lvl) => unsafe { nvim_mtitr_set_s_i(itr, *lvl, val) },
        }
    }

    fn add(&self, itr: MarkTreeIterHandle, delta: i32) {
        let v = self.get(itr);
        self.set(itr, v + delta);
    }
}

/// Delete mark at iterator position.
///
/// Returns the ID of the paired end mark (if any), or 0 if unpaired.
/// The iterator is updated to point at the key after the deleted one.
///
/// `rev` should be true if we plan to iterate backwards and delete stuff
/// before this key. Most of the time this is false.
#[must_use]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::many_single_char_names,
    clippy::too_many_lines,
    clippy::missing_panics_doc
)]
pub fn marktree_del_itr(b: MarkTreeHandle, itr: MarkTreeIterHandle, rev: bool) -> u64 {
    let mut adjustment: i32 = 0;

    let cur = unsafe { nvim_mtitr_get_x(itr) };
    let curi = unsafe { nvim_mtitr_get_i(itr) };
    let id = mt_lookup_key(&unsafe { nvim_mtnode_get_key(cur, curi) });

    // Step 1: Handle paired marks
    let raw = rawkey(itr);
    let mut other: u64 = 0;
    if mt_paired(&raw) && (raw.flags & MT_FLAG_ORPHANED == 0) {
        other = mt_lookup_key_side(&raw, !mt_end(&raw));
        let other_itr = unsafe { nvim_alloc_marktreeiter() };
        let _ = marktree_lookup(b, other, other_itr);
        rawkey_or_flags(other_itr, MT_FLAG_ORPHANED);
        // Remove intersect markers
        if mt_start(&raw) {
            let this_itr = unsafe { nvim_alloc_marktreeiter() };
            unsafe { nvim_marktree_itr_copy(this_itr, itr) };
            marktree_intersect_pair(b, id, this_itr, other_itr, true);
            unsafe { nvim_free_marktreeiter(this_itr) };
        } else {
            marktree_intersect_pair(b, other, other_itr, itr, true);
        }
        unsafe { nvim_free_marktreeiter(other_itr) };
    }

    // Step 2: If internal node, steal predecessor
    let cur_x = unsafe { nvim_mtitr_get_x(itr) };
    if unsafe { nvim_mtnode_get_level(cur_x) } != 0 {
        assert!(!rev, "marktree_del_itr: rev on internal node not supported");
        let _ = marktree_itr_prev(b, itr);
        adjustment = -1;
    }

    // Step 3: Delete key from leaf
    let x = unsafe { nvim_mtitr_get_x(itr) };
    debug_assert!(unsafe { nvim_mtnode_get_level(x) } == 0);
    let itr_i = unsafe { nvim_mtitr_get_i(itr) };
    let intkey = unsafe { nvim_mtnode_get_key(x, itr_i) };

    let mut meta_inc = meta_describe_key(&intkey);
    let x_n = unsafe { nvim_mtnode_get_n(x) };
    if x_n > itr_i + 1 {
        unsafe { nvim_mtnode_memmove_keys(x, itr_i, itr_i + 1, x_n - itr_i - 1) };
    }
    unsafe { nvim_mtnode_set_n(x, x_n - 1) };
    unsafe { nvim_marktree_dec_n_keys(b) };
    unsafe { nvim_marktree_del_id(b, id) };

    // Step 4: Replace internal key if we stole a predecessor
    if adjustment == -1 {
        let mut ilvl = unsafe { nvim_mtitr_get_lvl(itr) } - 1;
        let mut lnode = x;
        let mut start_id: u64 = 0;
        let mut did_bubble = false;
        let mut intkey_adj = intkey; // intkey with pos adjusted as we walk up
        if mt_end(&intkey) {
            start_id = mt_lookup_key_side(&intkey, false);
        }

        // Walk up from leaf to cur, adjusting intkey.pos and updating meta
        loop {
            let p = unsafe { nvim_mtnode_get_parent(lnode) };
            debug_assert!(ilvl >= 0, "ilvl went negative during del_itr walk");
            let i = unsafe { nvim_mtitr_get_s_i(itr, ilvl) };
            debug_assert!(unsafe { nvim_mtnode_get_ptr(p, i) } == lnode);
            if i > 0 {
                let prev_pos = unsafe { nvim_mtnode_get_key(p, i - 1) }.pos;
                unrelative(prev_pos, &mut intkey_adj.pos);
            }

            if p != cur && start_id != 0 {
                let ptr0 = unsafe { nvim_mtnode_get_ptr(p, 0) };
                if intersection_has(ptr0, start_id) {
                    // Undo the addition from the previous step if needed
                    let last = i32::from(lnode != x);
                    let p_n = unsafe { nvim_mtnode_get_n(p) };
                    for k in 0..p_n + last {
                        let child = unsafe { nvim_mtnode_get_ptr(p, k) };
                        unintersect_node(b, child, start_id, true);
                    }
                    intersect_node(b, p, start_id);
                    did_bubble = true;
                }
            }

            // p->meta[lnode->p_idx][m] -= meta_inc[m]
            let p_idx = unsafe { nvim_mtnode_get_p_idx(lnode) };
            for m in 0..K_MT_META_COUNT as i32 {
                let val = unsafe { nvim_mtnode_get_meta(p, p_idx, m) };
                unsafe { nvim_mtnode_set_meta(p, p_idx, m, val - meta_inc[m as usize]) };
            }

            lnode = p;
            ilvl -= 1;

            if lnode == cur {
                break;
            }
        }

        // Replace cur->key[curi] with intkey
        let deleted = unsafe { nvim_mtnode_get_key(cur, curi) };
        meta_inc = meta_describe_key(&deleted);
        unsafe { nvim_mtnode_set_key(cur, curi, intkey_adj) };
        marktree_refkey(b, cur, curi);

        // Check intersection for end marks
        if mt_end(&unsafe { nvim_mtnode_get_key(cur, curi) }) && !did_bubble {
            let pi = pseudo_index(x, 0); // sloppy pseudo-index
            let pi_start = pseudo_index_for_id(b, start_id, true);
            if pi_start > 0 && pi_start < pi {
                intersect_node(b, x, start_id);
            }
        }

        // Adjust positions of rightward subtree
        let mut rel_deleted = deleted;
        relative(intkey_adj.pos, &mut rel_deleted.pos);
        let mut y = unsafe { nvim_mtnode_get_ptr(cur, curi + 1) };
        if rel_deleted.pos.row != 0 || rel_deleted.pos.col != 0 {
            while !y.is_null() {
                let y_n = unsafe { nvim_mtnode_get_n(y) };
                for k in 0..y_n {
                    let mut yk = unsafe { nvim_mtnode_get_key(y, k) };
                    unrelative(rel_deleted.pos, &mut yk.pos);
                    unsafe { nvim_mtnode_set_key(y, k, yk) };
                }
                let y_level = unsafe { nvim_mtnode_get_level(y) };
                y = if y_level != 0 {
                    unsafe { nvim_mtnode_get_ptr(y, 0) }
                } else {
                    MTNodeHandle::null()
                };
            }
        }

        // itr->i--
        let cur_itr_i = unsafe { nvim_mtitr_get_i(itr) };
        unsafe { nvim_mtitr_set_i(itr, cur_itr_i - 1) };
    }

    // Propagate meta decrement up to root
    let mut lnode = cur;
    loop {
        let parent = unsafe { nvim_mtnode_get_parent(lnode) };
        if parent.is_null() {
            break;
        }
        let p_idx = unsafe { nvim_mtnode_get_p_idx(lnode) };
        for m in 0..K_MT_META_COUNT as i32 {
            let val = unsafe { nvim_mtnode_get_meta(parent, p_idx, m) };
            unsafe { nvim_mtnode_set_meta(parent, p_idx, m, val - meta_inc[m as usize]) };
        }
        lnode = parent;
    }
    // Update meta_root
    for m in 0..K_MT_META_COUNT as i32 {
        unsafe { nvim_marktree_sub_meta_root(b, m, meta_inc[m as usize]) };
    }

    // Step 5: Rebalance
    let mut itr_dirty = false;
    let mut rlvl = unsafe { nvim_mtitr_get_lvl(itr) } - 1;
    let mut lasti = LastiRef::ItrI;
    let mut ppos = unsafe { nvim_mtitr_get_pos(itr) };
    let root = unsafe { nvim_marktree_get_root(b) };
    let mut x_node = unsafe { nvim_mtitr_get_x(itr) };

    while x_node != root {
        debug_assert!(rlvl >= 0);
        let p = unsafe { nvim_mtnode_get_parent(x_node) };
        let x_n = unsafe { nvim_mtnode_get_n(x_node) };
        if x_n >= MT_BRANCH_FACTOR as i32 - 1 {
            // Node has enough keys, done
            break;
        }
        let pi = unsafe { nvim_mtitr_get_s_i(itr, rlvl) };
        debug_assert!(unsafe { nvim_mtnode_get_ptr(p, pi) } == x_node);
        if pi > 0 {
            let row_delta = unsafe { nvim_mtnode_get_key(p, pi - 1) }.pos.row;
            ppos.row -= row_delta;
            ppos.col = unsafe { nvim_mtitr_get_s_oldcol(itr, rlvl) };
        }
        // ppos is now the pos of p

        let left_n = if pi > 0 {
            let left_sib = unsafe { nvim_mtnode_get_ptr(p, pi - 1) };
            unsafe { nvim_mtnode_get_n(left_sib) }
        } else {
            0
        };
        let right_n = if pi < unsafe { nvim_mtnode_get_n(p) } {
            let right_sib = unsafe { nvim_mtnode_get_ptr(p, pi + 1) };
            unsafe { nvim_mtnode_get_n(right_sib) }
        } else {
            0
        };

        if pi > 0 && left_n > MT_BRANCH_FACTOR as i32 - 1 {
            // Steal from left: pivot_right
            lasti.add(itr, 1);
            itr_dirty = true;
            pivot_right(b, ppos, p, pi - 1);
            break;
        } else if pi < unsafe { nvim_mtnode_get_n(p) } && right_n > MT_BRANCH_FACTOR as i32 - 1 {
            // Steal from right: pivot_left
            pivot_left(b, ppos, p, pi);
            break;
        } else if pi > 0 {
            // Merge with left neighbour
            debug_assert!(left_n == MT_BRANCH_FACTOR as i32 - 1);
            lasti.add(itr, MT_BRANCH_FACTOR as i32);
            x_node = merge_node(b, p, pi - 1);
            // If lasti was ItrI, update itr->x to merged node
            if matches!(lasti, LastiRef::ItrI) {
                unsafe { nvim_mtitr_set_x(itr, x_node) };
            }
            // itr->s[rlvl].i--
            let s_i = unsafe { nvim_mtitr_get_s_i(itr, rlvl) };
            unsafe { nvim_mtitr_set_s_i(itr, rlvl, s_i - 1) };
            itr_dirty = true;
        } else {
            // Merge with right neighbour
            debug_assert!(
                pi < unsafe { nvim_mtnode_get_n(p) } && right_n == MT_BRANCH_FACTOR as i32 - 1
            );
            let _ = merge_node(b, p, pi);
            // no iter adjustment needed
        }

        lasti = LastiRef::S(rlvl);
        rlvl -= 1;
        x_node = p;
    }

    // Step 6: Root collapse
    let root = unsafe { nvim_marktree_get_root(b) };
    if !root.is_null() && unsafe { nvim_mtnode_get_n(root) } == 0 {
        let itr_lvl = unsafe { nvim_mtitr_get_lvl(itr) };
        if itr_lvl > 0 {
            // memmove(itr->s, itr->s + 1, (itr->lvl - 1) * sizeof(*itr->s))
            for j in 0..itr_lvl - 1 {
                let next_i = unsafe { nvim_mtitr_get_s_i(itr, j + 1) };
                let next_oldcol = unsafe { nvim_mtitr_get_s_oldcol(itr, j + 1) };
                unsafe { nvim_mtitr_set_s_i(itr, j, next_i) };
                unsafe { nvim_mtitr_set_s_oldcol(itr, j, next_oldcol) };
            }
            unsafe { nvim_mtitr_set_lvl(itr, itr_lvl - 1) };
        }
        if unsafe { nvim_mtnode_get_level(root) } != 0 {
            let oldroot = root;
            let new_root = unsafe { nvim_mtnode_get_ptr(oldroot, 0) };
            // assert meta_root matches oldroot->meta[0]
            unsafe { nvim_marktree_set_root(b, new_root) };
            unsafe { nvim_mtnode_set_parent(new_root, MTNodeHandle::null()) };
            marktree_free_node(b, oldroot);
        } else {
            // Tree is empty
            unsafe { nvim_mtitr_set_x(itr, MTNodeHandle::null()) };
        }
    }

    // Fix iterator position if needed
    let itr_x = unsafe { nvim_mtitr_get_x(itr) };
    if !itr_x.is_null() && itr_dirty {
        marktree_itr_fix_pos(b, itr);
    }

    // Bonus: Advance iterator to key after deleted one
    if adjustment == -1 {
        // We stand at the deleted space in previous leaf; skip stolen key and its replacement
        let _ = marktree_itr_next(b, itr);
        let _ = marktree_itr_next(b, itr);
    } else {
        let itr_x = unsafe { nvim_mtitr_get_x(itr) };
        if !itr_x.is_null() {
            let itr_i = unsafe { nvim_mtitr_get_i(itr) };
            let itr_x_n = unsafe { nvim_mtnode_get_n(itr_x) };
            if itr_i >= itr_x_n {
                // Deleted last key of leaf; go to inner key after it
                debug_assert!(unsafe { nvim_mtnode_get_level(itr_x) } == 0);
                let _ = marktree_itr_next(b, itr);
            }
        }
    }

    other
}

/// Revise meta counts after modifying a key's flags.
///
/// Call this after changing decoration flags on a key.
/// Walks the parent chain updating meta counts for the difference between
/// old_key and the current key at the iterator position.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
pub fn marktree_revise_meta(b: MarkTreeHandle, itr: MarkTreeIterHandle, old_key: MTKey) {
    let meta_old = meta_describe_key(&old_key);
    let cur_key = rawkey(itr);
    let meta_new = meta_describe_key(&cur_key);

    // Check if anything changed
    if meta_old == meta_new {
        return;
    }

    let mut lnode = unsafe { nvim_mtitr_get_x(itr) };
    while {
        let parent = unsafe { nvim_mtnode_get_parent(lnode) };
        !parent.is_null()
    } {
        let parent = unsafe { nvim_mtnode_get_parent(lnode) };
        let p_idx = unsafe { nvim_mtnode_get_p_idx(lnode) };
        for m in 0..K_MT_META_COUNT {
            let old_val = unsafe { nvim_mtnode_get_meta(parent, p_idx, m as i32) };
            let new_val = old_val.wrapping_add(meta_new[m]).wrapping_sub(meta_old[m]);
            unsafe { nvim_mtnode_set_meta(parent, p_idx, m as i32, new_val) };
        }
        lnode = parent;
    }

    for m in 0..K_MT_META_COUNT {
        unsafe {
            nvim_marktree_add_meta_root(b, m as i32, meta_new[m]);
            nvim_marktree_sub_meta_root(b, m as i32, meta_old[m]);
        }
    }
}

/// Move mark to a new position.
///
/// If the new position is within the same leaf node, an optimized
/// path is taken. Otherwise, delete and re-insert.
#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
pub fn marktree_move(b: MarkTreeHandle, itr: MarkTreeIterHandle, row: i32, col: i32) {
    let key = rawkey(itr);
    let x = unsafe { nvim_mtitr_get_x(itr) };
    let x_level = unsafe { nvim_mtnode_get_level(x) };

    if x_level == 0 {
        let mut internal = false;
        let mut newpos = MTPos { row, col };
        let x_parent = unsafe { nvim_mtnode_get_parent(x) };
        if x_parent.is_null() {
            // tree is one node, newpos is already "relative" to itr->pos
            internal = true;
        } else {
            let itr_pos = unsafe { nvim_mtitr_get_pos(itr) };
            // strictly _after_ key before `x`
            if pos_less(itr_pos, newpos) {
                relative(itr_pos, &mut newpos);
                let x_n = unsafe { nvim_mtnode_get_n(x) };
                let last_key = unsafe { nvim_mtnode_get_key(x, x_n - 1) };
                // strictly before the end of x
                if pos_less(newpos, last_key.pos) {
                    internal = true;
                }
            }
        }

        if internal {
            let itr_i = unsafe { nvim_mtitr_get_i(itr) };
            let cur_pos = unsafe { nvim_mtnode_get_key(x, itr_i) }.pos;
            if cur_pos.row == newpos.row && cur_pos.col == newpos.col {
                return;
            }
            let search_key = MTKey { pos: newpos, ..key };
            // Match C logic: new_i = rs_marktree_getp_aux(x, key, &match); if !match { new_i++; }
            let (raw_new_i, raw_match) = marktree_getp_aux(x, &search_key);
            let new_i = if raw_match { raw_new_i } else { raw_new_i + 1 };

            match new_i.cmp(&itr_i) {
                std::cmp::Ordering::Equal => {
                    unsafe { nvim_mtnode_set_key(x, itr_i, MTKey { pos: newpos, ..key }) };
                }
                std::cmp::Ordering::Less => {
                    unsafe {
                        nvim_mtnode_memmove_keys(x, new_i + 1, new_i, itr_i - new_i);
                    }
                    unsafe { nvim_mtnode_set_key(x, new_i, MTKey { pos: newpos, ..key }) };
                }
                std::cmp::Ordering::Greater => {
                    unsafe {
                        nvim_mtnode_memmove_keys(x, itr_i, itr_i + 1, new_i - itr_i - 1);
                    }
                    unsafe { nvim_mtnode_set_key(x, new_i - 1, MTKey { pos: newpos, ..key }) };
                }
            }
            return;
        }
    }

    let other = marktree_del_itr(b, itr, false);
    let new_key = MTKey {
        pos: MTPos { row, col },
        ..key
    };
    marktree_put_key(b, new_key);
    if other != 0 {
        marktree_restore_pair(b, new_key);
    }
    // itr might become invalid by put; mark x as null
    unsafe { nvim_mtitr_set_x(itr, MTNodeHandle::null()) };
}

/// Restore pair after move.
///
/// Re-establishes intersection markers for a paired mark.
pub fn marktree_restore_pair(b: MarkTreeHandle, key: MTKey) {
    let itr = unsafe { nvim_alloc_marktreeiter() };
    let end_itr = unsafe { nvim_alloc_marktreeiter() };

    let _ = marktree_lookup(b, mt_lookup_key_side(&key, false), itr);
    let _ = marktree_lookup(b, mt_lookup_key_side(&key, true), end_itr);

    let itr_valid = !unsafe { nvim_mtitr_get_x(itr) }.is_null();
    let end_itr_valid = !unsafe { nvim_mtitr_get_x(end_itr) }.is_null();

    if !itr_valid || !end_itr_valid {
        // Other end might be waiting to be restored later
        unsafe {
            nvim_free_marktreeiter(itr);
            nvim_free_marktreeiter(end_itr);
        }
        return;
    }

    rawkey_clear_flags(itr, MT_FLAG_ORPHANED);
    rawkey_clear_flags(end_itr, MT_FLAG_ORPHANED);

    let id = mt_lookup_key_side(&key, false);
    marktree_intersect_pair(b, id, itr, end_itr, false);

    unsafe {
        nvim_free_marktreeiter(itr);
        nvim_free_marktreeiter(end_itr);
    }
}

/// Delete key from id2node map.
pub fn marktree_del_id(b: MarkTreeHandle, id: u64) {
    unsafe { nvim_marktree_del_id(b, id) }
}

/// Decrement the number of keys in a marktree.
pub fn marktree_dec_n_keys(b: MarkTreeHandle) {
    unsafe { nvim_marktree_dec_n_keys(b) }
}

/// Subtract from meta_root by index.
pub fn marktree_sub_meta_root(b: MarkTreeHandle, m: i32, val: u32) {
    unsafe { nvim_marktree_sub_meta_root(b, m, val) }
}

// Note: rawkey() already defined above using mtnode_key()

/// Set flags on the raw key at iterator position.
pub fn rawkey_set_flags(itr: MarkTreeIterHandle, flags: u16) {
    unsafe { nvim_rawkey_set_flags(itr, flags) }
}

/// OR flags on the raw key at iterator position.
pub fn rawkey_or_flags(itr: MarkTreeIterHandle, flags: u16) {
    unsafe { nvim_rawkey_or_flags(itr, flags) }
}

/// AND-NOT flags on the raw key at iterator position.
pub fn rawkey_clear_flags(itr: MarkTreeIterHandle, flags: u16) {
    unsafe { nvim_rawkey_clear_flags(itr, flags) }
}

// ============================================================================
// Phase 7: Memory Management
// ============================================================================

/// Free a single node (destroy its intersection list, free memory, dec n_nodes).
pub fn marktree_free_node(b: MarkTreeHandle, x: MTNodeHandle) {
    unsafe {
        nvim_kvi_destroy_intersect(x);
        nvim_xfree_node(x);
        nvim_marktree_dec_n_nodes(b);
    }
}

/// Free an entire subtree recursively.
pub fn marktree_free_subtree(b: MarkTreeHandle, x: MTNodeHandle) {
    let x_level = unsafe { nvim_mtnode_get_level(x) };
    if x_level != 0 {
        let x_n = unsafe { nvim_mtnode_get_n(x) };
        for i in 0..=(x_n) {
            let child = unsafe { nvim_mtnode_get_ptr(x, i) };
            marktree_free_subtree(b, child);
        }
    }
    marktree_free_node(b, x);
}

/// Clear the entire marktree.
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
pub fn marktree_clear(b: MarkTreeHandle) {
    let root = unsafe { nvim_marktree_get_root(b) };
    if !root.is_null() {
        marktree_free_subtree(b, root);
        marktree_set_root(b, MTNodeHandle::null());
    }
    unsafe { nvim_marktree_destroy_id2node(b) };
    unsafe { nvim_marktree_set_n_keys(b, 0) };
    for m in 0..K_MT_META_COUNT {
        unsafe { nvim_marktree_set_meta_root(b, m as i32, 0) };
    }
    // assert!(b->n_nodes == 0) - validated by C clear function
}

// ============================================================================
// FFI Exports for Phase 5 & 7
// ============================================================================

/// Exported FFI version of `merge_node`.
#[no_mangle]
pub extern "C" fn rs_merge_node(b: MarkTreeHandle, p: MTNodeHandle, i: c_int) -> MTNodeHandle {
    merge_node(b, p, i)
}

/// Exported FFI version of `pivot_right`.
#[no_mangle]
pub extern "C" fn rs_pivot_right(b: MarkTreeHandle, p_pos: MTPos, p: MTNodeHandle, i: c_int) {
    pivot_right(b, p_pos, p, i);
}

/// Exported FFI version of `pivot_left`.
#[no_mangle]
pub extern "C" fn rs_pivot_left(b: MarkTreeHandle, p_pos: MTPos, p: MTNodeHandle, i: c_int) {
    pivot_left(b, p_pos, p, i);
}

/// Exported FFI version of `marktree_del_itr`.
#[no_mangle]
pub extern "C" fn rs_marktree_del_itr(
    b: MarkTreeHandle,
    itr: MarkTreeIterHandle,
    rev: bool,
) -> u64 {
    marktree_del_itr(b, itr, rev)
}

/// Exported FFI version of `marktree_revise_meta`.
#[no_mangle]
pub extern "C" fn rs_marktree_revise_meta(
    b: MarkTreeHandle,
    itr: MarkTreeIterHandle,
    old_key: MTKey,
) {
    marktree_revise_meta(b, itr, old_key);
}

/// Exported FFI version of `marktree_move`.
#[no_mangle]
pub extern "C" fn rs_marktree_move(
    b: MarkTreeHandle,
    itr: MarkTreeIterHandle,
    row: c_int,
    col: c_int,
) {
    marktree_move(b, itr, row, col);
}

/// Exported FFI version of `marktree_restore_pair`.
#[no_mangle]
pub extern "C" fn rs_marktree_restore_pair(b: MarkTreeHandle, key: MTKey) {
    marktree_restore_pair(b, key);
}

// Note: rs_rawkey already uses the existing rawkey() which calls mtnode_key()

/// Exported FFI version of `rawkey_set_flags`.
#[no_mangle]
pub extern "C" fn rs_rawkey_set_flags(itr: MarkTreeIterHandle, flags: u16) {
    rawkey_set_flags(itr, flags);
}

/// Exported FFI version of `rawkey_or_flags`.
#[no_mangle]
pub extern "C" fn rs_rawkey_or_flags(itr: MarkTreeIterHandle, flags: u16) {
    rawkey_or_flags(itr, flags);
}

/// Exported FFI version of `rawkey_clear_flags`.
#[no_mangle]
pub extern "C" fn rs_rawkey_clear_flags(itr: MarkTreeIterHandle, flags: u16) {
    rawkey_clear_flags(itr, flags);
}

/// Exported FFI version of `marktree_free_node`.
#[no_mangle]
pub extern "C" fn rs_marktree_free_node(b: MarkTreeHandle, x: MTNodeHandle) {
    marktree_free_node(b, x);
}

/// Exported FFI version of `marktree_free_subtree`.
#[no_mangle]
pub extern "C" fn rs_marktree_free_subtree(b: MarkTreeHandle, x: MTNodeHandle) {
    marktree_free_subtree(b, x);
}

/// Exported FFI version of `marktree_clear`.
#[no_mangle]
pub extern "C" fn rs_marktree_clear(b: MarkTreeHandle) {
    marktree_clear(b);
}

// ============================================================================
// Phase 6: Splice Operations
// ============================================================================

// ============================================================================
// Phase 6 Helpers: Damage tracking for intersection repair
// ============================================================================

/// Tracks a key that moved between nodes during splice so intersections can be repaired.
struct Damage {
    id: u64,
    old_node: MTNodeHandle,
    new_node: MTNodeHandle,
    old_i: c_int,
    new_i: c_int,
}

/// Check if two iterators point to the same key (same node and index).
fn itr_eq(itr1: MarkTreeIterHandle, itr2: MarkTreeIterHandle) -> bool {
    let x1 = unsafe { nvim_mtitr_get_x(itr1) };
    let x2 = unsafe { nvim_mtitr_get_x(itr2) };
    let i1 = unsafe { nvim_mtitr_get_i(itr1) };
    let i2 = unsafe { nvim_mtitr_get_i(itr2) };
    x1 == x2 && i1 == i2
}

/// Swap keys between two iterator positions. Keeps positions (pos) in place;
/// only the key identity (ns, id, flags, decor_data) is swapped. If the
/// iterators are on different nodes this may also need to fix parent meta counts.
///
/// `damage` collects pairs of (id, old_node, new_node) so that intersections
/// can be repaired after all moves are done.
#[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
fn swap_keys(
    b: MarkTreeHandle,
    itr1: MarkTreeIterHandle,
    itr2: MarkTreeIterHandle,
    damage: &mut Vec<Damage>,
) {
    let x1 = unsafe { nvim_mtitr_get_x(itr1) };
    let x2 = unsafe { nvim_mtitr_get_x(itr2) };
    let i1 = unsafe { nvim_mtitr_get_i(itr1) };
    let i2 = unsafe { nvim_mtitr_get_i(itr2) };

    if x1 != x2 {
        let key1 = rawkey(itr1);
        let key2 = rawkey(itr2);

        if mt_paired(&key1) {
            damage.push(Damage {
                id: mt_lookup_key(&key1),
                old_node: x1,
                new_node: x2,
                old_i: i1,
                new_i: i2,
            });
        }
        if mt_paired(&key2) {
            damage.push(Damage {
                id: mt_lookup_key(&key2),
                old_node: x2,
                new_node: x1,
                old_i: i2,
                new_i: i1,
            });
        }

        let meta_inc_1 = meta_describe_key(&key1);
        let meta_inc_2 = meta_describe_key(&key2);

        if meta_inc_1 != meta_inc_2 {
            let mut cur1 = x1;
            let mut cur2 = x2;
            while cur1 != cur2 {
                let level1 = unsafe { nvim_mtnode_get_level(cur1) };
                let level2 = unsafe { nvim_mtnode_get_level(cur2) };
                if level1 <= level2 {
                    // cur1 is not root; walk it up
                    let parent1 = unsafe { nvim_mtnode_get_parent(cur1) };
                    let p_idx1 = unsafe { nvim_mtnode_get_p_idx(cur1) };
                    for m in 0..K_MT_META_COUNT {
                        let old_val = unsafe { nvim_mtnode_get_meta(parent1, p_idx1, m as c_int) };
                        let new_val = old_val
                            .wrapping_add(meta_inc_2[m])
                            .wrapping_sub(meta_inc_1[m]);
                        unsafe { nvim_mtnode_set_meta(parent1, p_idx1, m as c_int, new_val) };
                    }
                    cur1 = parent1;
                }
                // Re-read levels after potentially updating cur1
                let level1_new = unsafe { nvim_mtnode_get_level(cur1) };
                let level2_new = unsafe { nvim_mtnode_get_level(cur2) };
                if level2_new < level1_new {
                    let parent2 = unsafe { nvim_mtnode_get_parent(cur2) };
                    let p_idx2 = unsafe { nvim_mtnode_get_p_idx(cur2) };
                    for m in 0..K_MT_META_COUNT {
                        let old_val = unsafe { nvim_mtnode_get_meta(parent2, p_idx2, m as c_int) };
                        let new_val = old_val
                            .wrapping_add(meta_inc_1[m])
                            .wrapping_sub(meta_inc_2[m]);
                        unsafe { nvim_mtnode_set_meta(parent2, p_idx2, m as c_int, new_val) };
                    }
                    cur2 = parent2;
                }
            }
        }
    }

    // Now swap the key identities, keeping positions in place.
    let key1 = rawkey(itr1);
    let key2 = rawkey(itr2);
    let pos1 = key1.pos;
    let pos2 = key2.pos;

    // Set itr1's key to key2's identity but keep pos1.
    let new_key1 = MTKey { pos: pos1, ..key2 };
    unsafe { nvim_mtnode_set_key(x1, i1, new_key1) };
    marktree_refkey(b, x1, i1);

    // Set itr2's key to key1's identity but keep pos2.
    let new_key2 = MTKey { pos: pos2, ..key1 };
    unsafe { nvim_mtnode_set_key(x2, i2, new_key2) };
    marktree_refkey(b, x2, i2);
}

/// Splice: handle text changes in buffer.
///
/// Updates mark positions based on text change:
/// - Right-gravity marks at deleted region edge stay at new_extent
/// - Marks in the deleted region are moved to start
/// - Marks after the deleted region are adjusted by delta
///
/// Returns true if any marks were moved.
#[must_use]
#[allow(clippy::too_many_lines)]
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
pub fn marktree_splice(
    b: MarkTreeHandle,
    start_line: i32,
    start_col: i32,
    old_extent_line: i32,
    old_extent_col: i32,
    new_extent_line: i32,
    new_extent_col: i32,
) -> bool {
    let start = MTPos {
        row: start_line,
        col: start_col,
    };
    let mut old_extent = MTPos {
        row: old_extent_line,
        col: old_extent_col,
    };
    let mut new_extent = MTPos {
        row: new_extent_line,
        col: new_extent_col,
    };

    let mut may_delete = old_extent.row != 0 || old_extent.col != 0;
    let same_line = old_extent.row == 0 && new_extent.row == 0;
    unrelative(start, &mut old_extent);
    unrelative(start, &mut new_extent);

    let mut oldbase = [MTPos { row: 0, col: 0 }; MT_MAX_DEPTH];

    let itr = unsafe { nvim_alloc_marktreeiter() };
    let _ = marktree_itr_get_ext_full(
        b,
        start,
        itr,
        false,
        true,
        oldbase.as_mut_ptr(),
        std::ptr::null(),
    );
    if unsafe { nvim_mtitr_get_x(itr) }.is_null() {
        unsafe { nvim_free_marktreeiter(itr) };
        return false;
    }

    let delta = MTPos {
        row: new_extent.row - old_extent.row,
        col: new_extent.col - old_extent.col,
    };

    let enditr = unsafe { nvim_alloc_marktreeiter() };

    if may_delete {
        let ipos = marktree_itr_pos(itr);
        if !pos_leq(old_extent, ipos)
            || (old_extent.row == ipos.row && old_extent.col == ipos.col && !mt_right(&rawkey(itr)))
        {
            let _ = marktree_itr_get_ext_full(
                b,
                old_extent,
                enditr,
                true,
                true,
                std::ptr::null_mut(),
                std::ptr::null(),
            );
        } else {
            may_delete = false;
        }
    }

    let mut past_right = false;
    let mut moved = false;
    let mut damage: Vec<Damage> = Vec::new();

    // Loop 1: move marks within deleted region to start position.
    // "oldbase" carries the information needed to calculate old position
    // of children.
    if may_delete {
        'loop1: while !unsafe { nvim_mtitr_get_x(itr) }.is_null() && !past_right {
            let itr_lvl = unsafe { nvim_mtitr_get_lvl(itr) } as usize;
            let itr_pos = unsafe { nvim_mtitr_get_pos(itr) };
            let mut loc_start = start;
            relative(itr_pos, &mut loc_start);
            let mut loc_old = old_extent;
            relative(oldbase[itr_lvl], &mut loc_old);

            // continue_same_node label -- inner loop for same leaf node
            loop {
                // NB: strictly should be less than the right gravity of loc_old, but
                // the iter comparison below will already break on that.
                if !pos_leq(rawkey(itr).pos, loc_old) {
                    break 'loop1;
                }

                if mt_right(&rawkey(itr)) {
                    while !itr_eq(itr, enditr) && mt_right(&rawkey(enditr)) {
                        let _ = marktree_itr_prev(b, enditr);
                    }
                    if mt_right(&rawkey(enditr)) {
                        // all remaining marks have right gravity
                        break 'loop1;
                    }
                    swap_keys(b, itr, enditr, &mut damage);
                }

                if itr_eq(itr, enditr) {
                    // actually, will be past_right after this key
                    past_right = true;
                }

                moved = true;
                let itr_x = unsafe { nvim_mtitr_get_x(itr) };
                let itr_level = unsafe { nvim_mtnode_get_level(itr_x) };
                if itr_level != 0 {
                    // internal node: update oldbase and skip subtree
                    let rkey_pos = rawkey(itr).pos;
                    oldbase[itr_lvl + 1] = rkey_pos;
                    unrelative(oldbase[itr_lvl], &mut oldbase[itr_lvl + 1]);
                    unsafe { nvim_rawkey_set_pos(itr, loc_start) };
                    let _ = marktree_itr_next_skip(
                        b,
                        itr,
                        false,
                        false,
                        oldbase.as_mut_ptr(),
                        std::ptr::null(),
                    );
                    break; // restart outer loop (loc_start/loc_old will be recomputed)
                }
                unsafe { nvim_rawkey_set_pos(itr, loc_start) };
                let itr_i = unsafe { nvim_mtitr_get_i(itr) };
                let itr_n = unsafe { nvim_mtnode_get_n(itr_x) };
                if itr_i < itr_n - 1 && !past_right {
                    unsafe { nvim_mtitr_set_i(itr, itr_i + 1) };
                    // goto continue_same_node: continue inner loop with same loc_start/loc_old
                } else if itr_i < itr_n - 1 {
                    // past_right: advance but exit inner loop
                    unsafe { nvim_mtitr_set_i(itr, itr_i + 1) };
                    break;
                } else {
                    let _ = marktree_itr_next(b, itr);
                    break;
                }
            }
        }

        // Loop 2: move marks just past the deleted region to new_extent.
        'loop2: while !unsafe { nvim_mtitr_get_x(itr) }.is_null() {
            let itr_lvl = unsafe { nvim_mtitr_get_lvl(itr) } as usize;
            let itr_pos = unsafe { nvim_mtitr_get_pos(itr) };
            let mut loc_new = new_extent;
            relative(itr_pos, &mut loc_new);
            let mut limit = old_extent;
            relative(oldbase[itr_lvl], &mut limit);

            // past_continue_same_node label -- inner loop for same leaf node
            loop {
                if pos_leq(limit, rawkey(itr).pos) {
                    break 'loop2;
                }

                let oldpos = rawkey(itr).pos;
                unsafe { nvim_rawkey_set_pos(itr, loc_new) };
                moved = true;

                let itr_x = unsafe { nvim_mtitr_get_x(itr) };
                let itr_level = unsafe { nvim_mtnode_get_level(itr_x) };
                if itr_level != 0 {
                    oldbase[itr_lvl + 1] = oldpos;
                    unrelative(oldbase[itr_lvl], &mut oldbase[itr_lvl + 1]);
                    let _ = marktree_itr_next_skip(
                        b,
                        itr,
                        false,
                        false,
                        oldbase.as_mut_ptr(),
                        std::ptr::null(),
                    );
                    break; // restart outer loop
                }
                let itr_i = unsafe { nvim_mtitr_get_i(itr) };
                let itr_n = unsafe { nvim_mtnode_get_n(itr_x) };
                if itr_i < itr_n - 1 {
                    unsafe { nvim_mtitr_set_i(itr, itr_i + 1) };
                    // goto past_continue_same_node: continue inner loop
                } else {
                    let _ = marktree_itr_next(b, itr);
                    break; // restart outer loop
                }
            }
        }
    }

    // Loop 3: adjust positions of remaining marks after the splice point.
    while !unsafe { nvim_mtitr_get_x(itr) }.is_null() {
        let itr_lvl = unsafe { nvim_mtitr_get_lvl(itr) } as usize;
        let itr_pos = unsafe { nvim_mtitr_get_pos(itr) };
        let mut pos = rawkey(itr).pos;
        unrelative(oldbase[itr_lvl], &mut pos);
        let realrow = pos.row;
        debug_assert!(realrow >= old_extent.row);
        let mut done = false;
        if realrow == old_extent.row {
            if delta.col != 0 {
                pos.col += delta.col;
            }
        } else if same_line {
            // optimization: column only adjustment can skip remaining rows
            done = true;
        }
        if delta.row != 0 {
            pos.row += delta.row;
            moved = true;
        }
        relative(itr_pos, &mut pos);
        unsafe { nvim_rawkey_set_pos(itr, pos) };
        if done {
            break;
        }
        let _ = marktree_itr_next_skip(b, itr, true, false, std::ptr::null_mut(), std::ptr::null());
    }

    // Damage repair: fix intersections for moved paired marks.
    if !damage.is_empty() {
        // Sort by id so start/end pairs are adjacent.
        damage.sort_by_key(|d| d.id);

        let mut i = 0usize;
        while i < damage.len() {
            debug_assert!(i == 0 || damage[i].id > damage[i - 1].id);
            if damage[i].id & MARKTREE_END_FLAG == 0 {
                // start mark
                if i + 1 < damage.len() && damage[i + 1].id == (damage[i].id | MARKTREE_END_FLAG) {
                    // pair: both start and end moved
                    let (d_part, rest) = damage.split_at(i + 1);
                    let d = &d_part[i];
                    let d2 = &rest[0];

                    let _ = marktree_itr_set_node(b, itr, d.old_node, d.old_i);
                    let _ = marktree_itr_set_node(b, enditr, d2.old_node, d2.old_i);
                    marktree_intersect_pair(b, d.id, itr, enditr, true);
                    let _ = marktree_itr_set_node(b, itr, d.new_node, d.new_i);
                    let _ = marktree_itr_set_node(b, enditr, d2.new_node, d2.new_i);
                    marktree_intersect_pair(b, d.id, itr, enditr, false);

                    i += 2; // consume two items
                    continue;
                }

                // lone start: end didn't move
                let d_id = damage[i].id;
                let d_old_node = damage[i].old_node;
                let d_old_i = damage[i].old_i;
                let d_new_node = damage[i].new_node;
                let d_new_i = damage[i].new_i;
                let endpos = unsafe { nvim_alloc_marktreeiter() };
                let _ = marktree_lookup(b, d_id | MARKTREE_END_FLAG, endpos);
                if !unsafe { nvim_mtitr_get_x(endpos) }.is_null() {
                    let _ = marktree_itr_set_node(b, itr, d_old_node, d_old_i);
                    unsafe { nvim_marktree_itr_copy(enditr, endpos) };
                    marktree_intersect_pair(b, d_id, itr, enditr, true);
                    let _ = marktree_itr_set_node(b, itr, d_new_node, d_new_i);
                    unsafe { nvim_marktree_itr_copy(enditr, endpos) };
                    marktree_intersect_pair(b, d_id, itr, enditr, false);
                }
                unsafe { nvim_free_marktreeiter(endpos) };
            } else {
                // lone end: start didn't move
                let d_id = damage[i].id;
                let d_old_node = damage[i].old_node;
                let d_old_i = damage[i].old_i;
                let d_new_node = damage[i].new_node;
                let d_new_i = damage[i].new_i;
                let start_id = d_id & !MARKTREE_END_FLAG;
                let startpos = unsafe { nvim_alloc_marktreeiter() };
                let _ = marktree_lookup(b, start_id, startpos);
                if !unsafe { nvim_mtitr_get_x(startpos) }.is_null() {
                    unsafe { nvim_marktree_itr_copy(itr, startpos) };
                    let _ = marktree_itr_set_node(b, enditr, d_old_node, d_old_i);
                    marktree_intersect_pair(b, start_id, itr, enditr, true);
                    unsafe { nvim_marktree_itr_copy(itr, startpos) };
                    let _ = marktree_itr_set_node(b, enditr, d_new_node, d_new_i);
                    marktree_intersect_pair(b, start_id, itr, enditr, false);
                }
                unsafe { nvim_free_marktreeiter(startpos) };
            }
            i += 1;
        }
    }

    unsafe {
        nvim_free_marktreeiter(itr);
        nvim_free_marktreeiter(enditr);
    }
    moved
}

/// Move region: move marks within a region to a new location.
///
/// Moves all marks in the region [start, start+extent) to [new, new+extent).
pub fn marktree_move_region(
    b: MarkTreeHandle,
    start_row: i32,
    start_col: i32,
    extent_row: i32,
    extent_col: i32,
    new_row: i32,
    new_col: i32,
) {
    let start = MTPos {
        row: start_row,
        col: start_col,
    };
    let size = MTPos {
        row: extent_row,
        col: extent_col,
    };
    let mut end = size;
    unrelative(start, &mut end);

    let itr = unsafe { nvim_alloc_marktreeiter() };
    let _ = marktree_itr_get_ext_full(
        b,
        start,
        itr,
        false,
        true,
        std::ptr::null_mut(),
        std::ptr::null(),
    );

    let mut saved: Vec<MTKey> = Vec::new();

    loop {
        let itr_x = unsafe { nvim_mtitr_get_x(itr) };
        if itr_x.is_null() {
            break;
        }
        let k = marktree_itr_current(itr);
        if !pos_leq(k.pos, end) || (k.pos.row == end.row && k.pos.col == end.col && mt_right(&k)) {
            break;
        }
        let mut saved_k = k;
        relative(start, &mut saved_k.pos);
        saved.push(saved_k);
        let _ = marktree_del_itr(b, itr, false);
    }

    unsafe { nvim_free_marktreeiter(itr) };

    let _ = marktree_splice(b, start.row, start.col, size.row, size.col, 0, 0);

    let new_pos = MTPos {
        row: new_row,
        col: new_col,
    };
    let _ = marktree_splice(b, new_pos.row, new_pos.col, 0, 0, size.row, size.col);

    for mut item in saved {
        unrelative(new_pos, &mut item.pos);
        marktree_put_key(b, item);
        if mt_paired(&item) {
            // Other end might be later in `saved`, this will safely bail out then
            marktree_restore_pair(b, item);
        }
    }
}

/// Test helper: delete both ends of a paired mark.
///
/// Used by unit tests to delete a mark pair by ns/id.
///
/// # Panics
///
/// Panics if the mark specified by `ns`/`id` is not a paired mark.
pub fn marktree_del_pair_test(b: MarkTreeHandle, ns: u32, id: u32) {
    let itr = unsafe { nvim_alloc_marktreeiter() };
    let _ = marktree_lookup_ns(b, ns, id, false, itr);
    let other = marktree_del_itr(b, itr, false);
    assert_ne!(other, 0, "marktree_del_pair_test: mark is not paired");
    let _ = marktree_lookup(b, other, itr);
    let _ = marktree_del_itr(b, itr, false);
    unsafe { nvim_free_marktreeiter(itr) };
}

// ============================================================================
// Phase 8: Debug and Validation
// ============================================================================

/// Recursive node validator.
///
/// Returns the total number of keys in the subtree rooted at `x`.
///
/// # Panics
///
/// Panics if any B-tree invariant is violated.
#[allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::missing_panics_doc
)]
fn marktree_check_node_handle(
    b: MarkTreeHandle,
    x: MTNodeHandle,
    root: MTNodeHandle,
    last: &mut MTPos,
    last_right: &mut bool,
    meta_node_ref: &[u32; K_MT_META_COUNT],
) -> usize {
    let n = unsafe { nvim_mtnode_get_n(x) } as usize;
    let level = unsafe { nvim_mtnode_get_level(x) };
    let ni = n as c_int;

    assert!(n < 2 * MT_BRANCH_FACTOR, "node has too many keys: {n}");
    if x != root {
        assert!(
            n >= MT_BRANCH_FACTOR - 1,
            "non-root node has too few keys: {n}"
        );
    }

    let mut count = n;

    for i in 0..ni {
        let iu = i as usize;
        if level != 0 {
            let child = unsafe { nvim_mtnode_get_ptr(x, i) };
            let child_meta = mtnode_meta(x, i);
            count += marktree_check_node_handle(b, child, root, last, last_right, &child_meta);
        } else {
            *last = MTPos { row: 0, col: 0 };
        }
        if iu > 0 {
            let prev_key = unsafe { nvim_mtnode_get_key(x, i - 1) };
            unrelative(prev_key.pos, last);
        }
        let key = unsafe { nvim_mtnode_get_key(x, i) };
        assert!(
            pos_leq(*last, key.pos),
            "key ordering violated at index {iu}: last={last:?} key={:?}",
            key.pos
        );
        if last.row == key.pos.row && last.col == key.pos.col {
            assert!(
                !*last_right || mt_right(&key),
                "right-gravity ordering violated at index {iu}"
            );
        }
        *last_right = mt_right(&key);
        assert!(key.pos.col >= 0, "key has negative col at index {iu}");
        let looked_up_node = unsafe { nvim_marktree_id2node(b, mt_lookup_key(&key)) };
        assert!(
            looked_up_node == x,
            "id2node mismatch at key index {iu}: expected {x:?}, got {looked_up_node:?}"
        );
    }

    if level != 0 {
        let last_child = unsafe { nvim_mtnode_get_ptr(x, ni) };
        let last_child_meta = mtnode_meta(x, ni);
        count +=
            marktree_check_node_handle(b, last_child, root, last, last_right, &last_child_meta);
        let last_key = unsafe { nvim_mtnode_get_key(x, ni - 1) };
        unrelative(last_key.pos, last);

        for i in 0..=ni {
            let child = unsafe { nvim_mtnode_get_ptr(x, i) };
            let child_parent = unsafe { nvim_mtnode_get_parent(child) };
            let child_p_idx = unsafe { nvim_mtnode_get_p_idx(child) };
            let child_level = unsafe { nvim_mtnode_get_level(child) };
            assert!(child_parent == x, "child {i} has wrong parent");
            assert!(child_p_idx == i, "child {i} has wrong p_idx: {child_p_idx}");
            assert!(
                child_level == level - 1,
                "child {i} has wrong level: {child_level} (expected {})",
                level - 1
            );
            // PARANOIA: check no double node ref
            for j in 0..i {
                let other_child = unsafe { nvim_mtnode_get_ptr(x, j) };
                assert!(
                    child != other_child,
                    "duplicate child pointer at {i} and {j}"
                );
            }
        }
    } else if n > 0 {
        let last_key = unsafe { nvim_mtnode_get_key(x, ni - 1) };
        *last = last_key.pos;
    }

    let meta_node = meta_describe_node(x);
    for m in 0..K_MT_META_COUNT {
        assert!(
            meta_node_ref[m] == meta_node[m],
            "meta mismatch at m={m}: ref={} got={}",
            meta_node_ref[m],
            meta_node[m]
        );
    }

    count
}

/// Check marktree invariants.
///
/// Validates the B-tree structure, intersection markers, and meta counts.
///
/// # Panics
///
/// Panics if any invariant is violated.
pub fn marktree_check(b: MarkTreeHandle) {
    let root = unsafe { nvim_marktree_get_root(b) };
    if root.is_null() {
        let n_keys = unsafe { nvim_marktree_get_n_keys(b) };
        assert!(n_keys == 0, "empty tree has non-zero n_keys: {n_keys}");
        let id2node_count = unsafe { nvim_marktree_id2node_count(b) };
        assert!(
            id2node_count == 0,
            "empty tree has non-zero id2node count: {id2node_count}"
        );
        return;
    }

    let mut last = MTPos { row: 0, col: 0 };
    let mut last_right = false;
    let meta_root = marktree_meta_root(b);
    let counted = marktree_check_node_handle(b, root, root, &mut last, &mut last_right, &meta_root);
    let stored = unsafe { nvim_marktree_get_n_keys(b) };
    assert!(
        stored == counted,
        "n_keys mismatch: stored={stored}, counted={counted}"
    );
    let id2node_count = unsafe { nvim_marktree_id2node_count(b) };
    assert!(
        stored == id2node_count,
        "n_keys ({stored}) != id2node count ({id2node_count})"
    );
}

/// Recursively save and clear intersection lists for all nodes in the subtree.
///
/// Saves each non-empty intersection list (with a sentinel `u64::MAX`) into `saved`,
/// then clears the node's intersection list.
fn mt_recurse_nodes_handle(
    x: MTNodeHandle,
    saved: &mut std::collections::HashMap<usize, Vec<u64>>,
) {
    let size = unsafe { nvim_mtnode_get_intersect_size(x) };
    if size > 0 {
        let mut list = Vec::with_capacity(size + 1);
        for i in 0..size {
            list.push(unsafe { nvim_mtnode_get_intersect_elem(x, i) });
        }
        list.push(u64::MAX); // sentinel
        saved.insert(x.0 as usize, list);
        unsafe { nvim_mtnode_intersect_clear(x) };
    }

    let level = unsafe { nvim_mtnode_get_level(x) };
    if level != 0 {
        let n = unsafe { nvim_mtnode_get_n(x) };
        for i in 0..=n {
            let child = unsafe { nvim_mtnode_get_ptr(x, i) };
            mt_recurse_nodes_handle(child, saved);
        }
    }
}

/// Recursively compare rebuilt intersection lists against saved ones.
///
/// Returns `true` if all nodes match, `false` otherwise.
fn mt_recurse_nodes_compare_handle(
    x: MTNodeHandle,
    saved: &std::collections::HashMap<usize, Vec<u64>>,
) -> bool {
    let current_size = unsafe { nvim_mtnode_get_intersect_size(x) };

    match saved.get(&(x.0 as usize)) {
        Some(ref_list) => {
            // ref_list ends with u64::MAX sentinel; check element-by-element
            let mut i = 0usize;
            loop {
                let sentinel = ref_list[i] == u64::MAX;
                if sentinel {
                    if i != current_size {
                        return false;
                    }
                    break;
                }
                if current_size <= i {
                    return false;
                }
                let cur_elem = unsafe { nvim_mtnode_get_intersect_elem(x, i) };
                if ref_list[i] != cur_elem {
                    return false;
                }
                i += 1;
            }
        }
        None => {
            if current_size != 0 {
                return false;
            }
        }
    }

    let level = unsafe { nvim_mtnode_get_level(x) };
    if level != 0 {
        let n = unsafe { nvim_mtnode_get_n(x) };
        for i in 0..=n {
            let child = unsafe { nvim_mtnode_get_ptr(x, i) };
            if !mt_recurse_nodes_compare_handle(child, saved) {
                return false;
            }
        }
    }

    true
}

/// Validate intersection markers by rebuilding them from scratch and comparing.
///
/// Returns `true` if all intersection lists are consistent, `false` otherwise.
///
/// # Panics
///
/// Does not panic (returns false on mismatch).
#[must_use]
#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
pub fn marktree_check_intersections(b: MarkTreeHandle) -> bool {
    let root = unsafe { nvim_marktree_get_root(b) };
    if root.is_null() {
        return true;
    }

    // Step 1: save and clear all intersection lists
    let mut saved: std::collections::HashMap<usize, Vec<u64>> = std::collections::HashMap::new();
    mt_recurse_nodes_handle(root, &mut saved);

    // Step 2: iterate over all marks; for each start mark of a pair,
    // rebuild intersections
    let itr = unsafe { nvim_alloc_marktreeiter() };
    let _ = marktree_itr_first(b, itr);
    loop {
        let mark = marktree_itr_current(itr);
        if mark.pos.row < 0 {
            break;
        }

        if mt_start(&mark) {
            let end_id = mt_lookup_id(mark.ns, mark.id, true);
            let end_itr = unsafe { nvim_alloc_marktreeiter() };
            let k = marktree_lookup(b, end_id, end_itr);
            if k.pos.row >= 0 {
                let start_itr = unsafe { nvim_alloc_marktreeiter() };
                // Copy current iterator to start_itr (equivalent to `*start_itr = *itr`)
                unsafe { nvim_marktree_itr_copy(start_itr, itr) };
                marktree_intersect_pair(b, mt_lookup_key(&mark), start_itr, end_itr, false);
                unsafe { nvim_free_marktreeiter(start_itr) };
            }
            unsafe { nvim_free_marktreeiter(end_itr) };
        }

        let _ = marktree_itr_next(b, itr);
    }
    unsafe { nvim_free_marktreeiter(itr) };

    // Step 3: compare rebuilt intersections against saved ones
    let status = mt_recurse_nodes_compare_handle(root, &saved);

    // Step 4: restore original intersection lists
    restore_intersections(root, &saved);

    status
}

/// Restore saved intersection lists to all nodes.
fn restore_intersections(x: MTNodeHandle, saved: &std::collections::HashMap<usize, Vec<u64>>) {
    unsafe { nvim_mtnode_intersect_clear(x) };
    if let Some(list) = saved.get(&(x.0 as usize)) {
        // list ends with u64::MAX sentinel; push all elements before sentinel
        for &id in list.iter().take_while(|&&v| v != u64::MAX) {
            unsafe { nvim_mtnode_intersect_push(x, id) };
        }
    }

    let level = unsafe { nvim_mtnode_get_level(x) };
    if level != 0 {
        let n = unsafe { nvim_mtnode_get_n(x) };
        for i in 0..=n {
            let child = unsafe { nvim_mtnode_get_ptr(x, i) };
            restore_intersections(child, saved);
        }
    }
}

// ============================================================================
// FFI Exports for Phase 6 & 8
// ============================================================================

/// Exported FFI version of `marktree_splice`.
#[no_mangle]
pub extern "C" fn rs_marktree_splice(
    b: MarkTreeHandle,
    start_line: i32,
    start_col: c_int,
    old_extent_line: c_int,
    old_extent_col: c_int,
    new_extent_line: c_int,
    new_extent_col: c_int,
) -> bool {
    marktree_splice(
        b,
        start_line,
        start_col,
        old_extent_line,
        old_extent_col,
        new_extent_line,
        new_extent_col,
    )
}

/// Exported FFI version of `marktree_move_region`.
#[no_mangle]
pub extern "C" fn rs_marktree_move_region(
    b: MarkTreeHandle,
    start_row: c_int,
    start_col: c_int,
    extent_row: c_int,
    extent_col: c_int,
    new_row: c_int,
    new_col: c_int,
) {
    marktree_move_region(
        b, start_row, start_col, extent_row, extent_col, new_row, new_col,
    );
}

/// Exported FFI version of `marktree_check`.
#[no_mangle]
pub extern "C" fn rs_marktree_check(b: MarkTreeHandle) {
    marktree_check(b);
}

/// Exported FFI version of `marktree_check_intersections`.
#[no_mangle]
pub extern "C" fn rs_marktree_check_intersections(b: MarkTreeHandle) -> bool {
    marktree_check_intersections(b)
}

// ============================================================================
// Phase 1 (pass 2): Test Helpers
// ============================================================================

/// Delete both ends of a paired mark by ns/id. For unit tests.
#[no_mangle]
pub extern "C" fn rs_marktree_del_pair_test(b: MarkTreeHandle, ns: u32, id: u32) {
    marktree_del_pair_test(b, ns, id);
}

// ============================================================================
// Phase 9: Test Helpers
// ============================================================================

/// Insert a mark for unit tests, ported from C `marktree_put_test`.
#[no_mangle]
pub extern "C" fn rs_marktree_put_test(
    b: MarkTreeHandle,
    ns: u32,
    id: u32,
    row: c_int,
    col: c_int,
    right_gravity: bool,
    end_row: c_int,
    end_col: c_int,
    end_right: bool,
    meta_inline: bool,
) {
    let mut flags = mt_flags(right_gravity, false, false, false);
    // The specific choice is irrelevant here, we pick one counted decor
    // type to test the counting and filtering logic.
    if meta_inline {
        flags |= MT_FLAG_DECOR_VIRT_TEXT_INLINE;
    }
    let key = MTKey {
        pos: MTPos { row, col },
        ns,
        id,
        flags,
        decor_data: DecorInlineData::zero(),
    };
    marktree_put(b, key, end_row, end_col, end_right);
}

/// Check right gravity for unit tests, ported from C `mt_right_test`.
#[no_mangle]
pub extern "C" fn rs_mt_right_test(key: MTKey) -> bool {
    mt_right(&key)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pos_comparison() {
        let a = MTPos::new(0, 0);
        let b = MTPos::new(0, 5);
        let c = MTPos::new(1, 0);
        let d = MTPos::new(1, 5);

        // pos_leq tests
        assert!(pos_leq(a, a)); // Equal
        assert!(pos_leq(a, b)); // Same row, less col
        assert!(pos_leq(a, c)); // Less row
        assert!(pos_leq(b, c)); // Less row (col doesn't matter)
        assert!(pos_leq(c, d)); // Same row, less col
        assert!(!pos_leq(b, a)); // Same row, greater col
        assert!(!pos_leq(c, a)); // Greater row

        // pos_less tests
        assert!(!pos_less(a, a)); // Equal is not less
        assert!(pos_less(a, b));
        assert!(pos_less(a, c));
        assert!(pos_less(b, c));
        assert!(!pos_less(b, a));
    }

    #[test]
    fn test_relative_unrelative() {
        // Test relative positioning on same row
        let base = MTPos::new(5, 10);
        let mut val = MTPos::new(5, 15);
        relative(base, &mut val);
        assert_eq!(val, MTPos::new(0, 5));

        // Undo with unrelative
        unrelative(base, &mut val);
        assert_eq!(val, MTPos::new(5, 15));

        // Test relative positioning on different rows
        let mut val2 = MTPos::new(8, 3);
        relative(base, &mut val2);
        assert_eq!(val2, MTPos::new(3, 3));

        unrelative(base, &mut val2);
        assert_eq!(val2, MTPos::new(8, 3));
    }

    #[test]
    fn test_compose() {
        // Compose on same row
        let mut base = MTPos::new(5, 10);
        let val = MTPos::new(0, 5);
        compose(&mut base, val);
        assert_eq!(base, MTPos::new(5, 15));

        // Compose with row change
        let mut base2 = MTPos::new(5, 10);
        let val2 = MTPos::new(3, 7);
        compose(&mut base2, val2);
        assert_eq!(base2, MTPos::new(8, 7));
    }

    #[test]
    fn test_mt_lookup_id() {
        let ns = 1u32;
        let id = 100u32;

        let start_id = mt_lookup_id(ns, id, false);
        let end_id = mt_lookup_id(ns, id, true);

        // End ID should be start ID + 1
        assert_eq!(end_id, start_id | MARKTREE_END_FLAG);
        assert_ne!(start_id, end_id);

        // Different ns/id should give different results
        let other_id = mt_lookup_id(2, 100, false);
        assert_ne!(start_id, other_id);
    }

    #[test]
    fn test_flag_checks() {
        let mut key = MTKey::default();

        // Test paired flag
        assert!(!mt_paired(&key));
        key.flags |= MT_FLAG_PAIRED;
        assert!(mt_paired(&key));

        // Test end flag
        assert!(!mt_end(&key));
        key.flags |= MT_FLAG_END;
        assert!(mt_end(&key));

        // Test start (paired but not end)
        key.flags = MT_FLAG_PAIRED;
        assert!(mt_start(&key));
        key.flags |= MT_FLAG_END;
        assert!(!mt_start(&key)); // End of pair is not start

        // Test right gravity
        key.flags = 0;
        assert!(!mt_right(&key));
        key.flags = MT_FLAG_RIGHT_GRAVITY;
        assert!(mt_right(&key));
    }

    #[test]
    fn test_key_cmp() {
        let k1 = MTKey {
            pos: MTPos::new(0, 0),
            ns: 0,
            id: 0,
            flags: MT_FLAG_REAL,
            decor_data: DecorInlineData::zero(),
        };
        let k2 = MTKey {
            pos: MTPos::new(0, 5),
            ns: 0,
            id: 0,
            flags: MT_FLAG_REAL,
            decor_data: DecorInlineData::zero(),
        };
        let k3 = MTKey {
            pos: MTPos::new(1, 0),
            ns: 0,
            id: 0,
            flags: MT_FLAG_REAL,
            decor_data: DecorInlineData::zero(),
        };

        // Row comparison
        assert!(key_cmp(&k1, &k3) < 0);
        assert!(key_cmp(&k3, &k1) > 0);

        // Column comparison
        assert!(key_cmp(&k1, &k2) < 0);
        assert!(key_cmp(&k2, &k1) > 0);

        // Equal keys
        assert_eq!(key_cmp(&k1, &k1), 0);
    }

    #[test]
    fn test_mt_flags() {
        assert_eq!(mt_flags(false, false, false, false), 0);
        assert_eq!(mt_flags(true, false, false, false), MT_FLAG_RIGHT_GRAVITY);
        assert_eq!(mt_flags(false, true, false, false), MT_FLAG_NO_UNDO);
        assert_eq!(mt_flags(false, false, true, false), MT_FLAG_INVALIDATE);
        assert_eq!(mt_flags(false, false, false, true), MT_FLAG_DECOR_EXT);
        assert_eq!(
            mt_flags(true, true, true, true),
            MT_FLAG_RIGHT_GRAVITY | MT_FLAG_NO_UNDO | MT_FLAG_INVALIDATE | MT_FLAG_DECOR_EXT
        );
    }

    #[test]
    fn test_meta_describe_key() {
        // Empty key - no decoration
        let key = MTKey::default();
        let meta = meta_describe_key(&key);
        assert_eq!(meta, [0, 0, 0, 0, 0]);

        // Key with inline virt text
        let key_inline = MTKey {
            flags: flags::MT_FLAG_DECOR_VIRT_TEXT_INLINE,
            ..Default::default()
        };
        let meta_inline = meta_describe_key(&key_inline);
        assert_eq!(meta_inline[K_MT_META_INLINE], 1);
        assert_eq!(meta_inline[K_MT_META_LINES], 0);

        // Key with virt lines
        let key_lines = MTKey {
            flags: flags::MT_FLAG_DECOR_VIRT_LINES,
            ..Default::default()
        };
        let meta_lines = meta_describe_key(&key_lines);
        assert_eq!(meta_lines[K_MT_META_LINES], 1);
        assert_eq!(meta_lines[K_MT_META_INLINE], 0);

        // End key - should not count
        let key_end = MTKey {
            flags: flags::MT_FLAG_DECOR_VIRT_TEXT_INLINE | MT_FLAG_END,
            ..Default::default()
        };
        let meta_end = meta_describe_key(&key_end);
        assert_eq!(meta_end, [0, 0, 0, 0, 0]);

        // Invalid key - should not count
        let key_invalid = MTKey {
            flags: flags::MT_FLAG_DECOR_VIRT_TEXT_INLINE | MT_FLAG_INVALID,
            ..Default::default()
        };
        let meta_invalid = meta_describe_key(&key_invalid);
        assert_eq!(meta_invalid, [0, 0, 0, 0, 0]);
    }
}
