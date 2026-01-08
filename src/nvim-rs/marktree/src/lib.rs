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
pub mod insert;
pub mod intersection;
pub mod iter;
pub mod node;
pub mod overlap;
pub mod splice;

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

/// Key for a mark in the tree.
///
/// The `decor_data` field is represented as a u64 to match the C union size.
/// The actual interpretation depends on the MT_FLAG_DECOR_EXT flag.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MTKey {
    pub pos: MTPos,
    pub ns: u32,
    pub id: u32,
    pub flags: u16,
    /// Decoration data (union in C: DecorHighlightInline or DecorExt).
    /// Stored as raw bytes for FFI compatibility.
    pub decor_data: u64,
}

impl Default for MTKey {
    fn default() -> Self {
        Self {
            pos: MTPos::new(-1, -1),
            ns: 0,
            id: 0,
            flags: 0,
            decor_data: 0,
        }
    }
}

impl MTKey {
    /// Create an invalid key sentinel.
    #[inline]
    #[must_use]
    pub const fn invalid() -> Self {
        Self {
            pos: MTPos { row: -1, col: -1 },
            ns: 0,
            id: 0,
            flags: 0,
            decor_data: 0,
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
    fn nvim_marktree_lookup(b: MarkTreeHandle, id: u64, itr: MarkTreeIterHandle) -> MTKey;

    /// Lookup a mark by namespace and ID.
    fn nvim_marktree_lookup_ns(
        b: MarkTreeHandle,
        ns: u32,
        id: u32,
        end: bool,
        itr: MarkTreeIterHandle,
    ) -> MTKey;

    /// Get the alternate end of a paired mark.
    fn nvim_marktree_get_alt(b: MarkTreeHandle, mark: MTKey, itr: MarkTreeIterHandle) -> MTKey;

    /// Get the position of the alternate end of a paired mark.
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

    // ========================================================================
    // Helper Functions
    // ========================================================================

    /// Compute pseudo-index for position (x, i).
    fn nvim_pseudo_index(x: MTNodeHandle, i: c_int) -> u64;

    /// Compute pseudo-index for a mark ID.
    fn nvim_pseudo_index_for_id(b: MarkTreeHandle, id: u64, sloppy: bool) -> u64;

    /// Set iterator to point at node n, index i.
    fn nvim_marktree_itr_set_node(
        b: MarkTreeHandle,
        itr: MarkTreeIterHandle,
        n: MTNodeHandle,
        i: c_int,
    ) -> MTKey;

    /// Fix iterator position after setting node directly.
    fn nvim_marktree_itr_fix_pos(b: MarkTreeHandle, itr: MarkTreeIterHandle);

    /// Describe meta counts for a key incrementally.
    #[allow(dead_code)]
    fn nvim_meta_describe_key_inc(meta_inc: *mut u32, k: *mut MTKey);

    /// Describe meta counts for a whole node.
    #[allow(dead_code)]
    fn nvim_meta_describe_node(meta_node: *mut u32, x: MTNodeHandle);

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

    /// Add an intersection to a node (sorted insert).
    fn nvim_intersect_node(b: MarkTreeHandle, x: MTNodeHandle, id: u64);

    /// Remove an intersection from a node.
    fn nvim_unintersect_node(b: MarkTreeHandle, x: MTNodeHandle, id: u64, strict: bool);

    /// Copy intersections from one node to another.
    fn nvim_kvi_copy_intersect(dst: MTNodeHandle, src: MTNodeHandle);

    /// Clear intersections in a node.
    #[allow(dead_code)]
    fn nvim_kvi_init_intersect(x: MTNodeHandle);

    /// Check if a node's intersect list contains the given ID.
    fn nvim_intersection_has(x: MTNodeHandle, id: u64) -> bool;

    // ========================================================================
    // B-tree Operations (Phase 4)
    // ========================================================================

    /// Split a full child node during insertion.
    fn nvim_split_node(b: MarkTreeHandle, x: MTNodeHandle, i: c_int, next: MTKey);

    /// Recursive insertion helper.
    fn nvim_marktree_putp_aux(b: MarkTreeHandle, x: MTNodeHandle, k: MTKey, meta_inc: *mut u32);

    /// Insert a key into the marktree.
    fn nvim_marktree_put_key(b: MarkTreeHandle, k: MTKey);

    /// Insert a mark with optional paired end.
    fn nvim_marktree_put(
        b: MarkTreeHandle,
        key: MTKey,
        end_row: c_int,
        end_col: c_int,
        end_right: bool,
    );

    /// Mark intersections between paired marks.
    fn nvim_marktree_intersect_pair(
        b: MarkTreeHandle,
        id: u64,
        itr: MarkTreeIterHandle,
        end_itr: MarkTreeIterHandle,
        delete: bool,
    );

    /// Bubble up common intersections to parent.
    fn nvim_bubble_up(x: MTNodeHandle);

    // ========================================================================
    // B-tree Deletion Operations (Phase 5)
    // ========================================================================

    /// Delete mark at iterator position.
    fn nvim_marktree_del_itr(b: MarkTreeHandle, itr: MarkTreeIterHandle, rev: bool) -> u64;

    /// Revise meta counts after key modification.
    fn nvim_marktree_revise_meta(b: MarkTreeHandle, itr: MarkTreeIterHandle, old_key: MTKey);

    /// Move mark to a new position.
    fn nvim_marktree_move(b: MarkTreeHandle, itr: MarkTreeIterHandle, row: c_int, col: c_int);

    /// Restore pair after move.
    fn nvim_marktree_restore_pair(b: MarkTreeHandle, key: MTKey);

    /// Pivot right (steal from left sibling).
    fn nvim_pivot_right(b: MarkTreeHandle, p_pos: MTPos, p: MTNodeHandle, i: c_int);

    /// Pivot left (steal from right sibling).
    fn nvim_pivot_left(b: MarkTreeHandle, p_pos: MTPos, p: MTNodeHandle, i: c_int);

    /// Merge two nodes.
    fn nvim_merge_node(b: MarkTreeHandle, p: MTNodeHandle, i: c_int) -> MTNodeHandle;

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

    // ========================================================================
    // Memory Management Operations (Phase 7)
    // ========================================================================

    /// Free a single node.
    fn nvim_marktree_free_node(b: MarkTreeHandle, x: MTNodeHandle);

    /// Free an entire subtree.
    fn nvim_marktree_free_subtree(b: MarkTreeHandle, x: MTNodeHandle);

    /// Clear the entire marktree.
    fn nvim_marktree_clear(b: MarkTreeHandle);

    // ========================================================================
    // Splice Operations (Phase 6)
    // ========================================================================

    /// Splice: handle text changes in buffer.
    fn nvim_marktree_splice(
        b: MarkTreeHandle,
        start_line: i32,
        start_col: c_int,
        old_extent_line: c_int,
        old_extent_col: c_int,
        new_extent_line: c_int,
        new_extent_col: c_int,
    ) -> bool;

    /// Move region: move marks within a region to a new location.
    fn nvim_marktree_move_region(
        b: MarkTreeHandle,
        start_row: c_int,
        start_col: c_int,
        extent_row: c_int,
        extent_col: c_int,
        new_row: c_int,
        new_col: c_int,
    );

    // ========================================================================
    // Debug and Validation (Phase 8)
    // ========================================================================

    /// Check marktree invariants.
    fn nvim_marktree_check(b: MarkTreeHandle);
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

/// Extended version of iterator positioning.
///
/// If `last` is true, position at the last key <= position.
/// If `gravity` is true, consider right gravity when positioning.
#[allow(clippy::many_single_char_names)] // Matching C code naming conventions
#[must_use]
pub fn marktree_itr_get_ext(
    b: MarkTreeHandle,
    p: MTPos,
    itr: MarkTreeIterHandle,
    last: bool,
    gravity: bool,
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
        decor_data: 0,
    };

    unsafe {
        nvim_mtitr_set_pos(itr, MTPos::new(0, 0));
        nvim_mtitr_set_lvl(itr, 0);
    }

    let mut x = marktree_root(b);
    let mut current_pos = MTPos::new(0, 0);
    let mut lvl = 0;

    loop {
        let (i, _) = marktree_getp_aux(x, &k);
        let i = i + 1; // marktree_getp_aux returns position before, we want after

        if mtnode_level(x) == 0 {
            unsafe {
                nvim_mtitr_set_x(itr, x);
                nvim_mtitr_set_i(itr, i);
                nvim_mtitr_set_lvl(itr, lvl);
                nvim_mtitr_set_pos(itr, current_pos);
            }
            break;
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
    }

    if last {
        marktree_itr_prev(b, itr)
    } else {
        let i = unsafe { nvim_mtitr_get_i(itr) };
        let x = unsafe { nvim_mtitr_get_x(itr) };
        if i >= mtnode_n(x) {
            // Need to go to next internal key
            marktree_itr_next(b, itr)
        } else {
            true
        }
    }
}

/// Exported FFI version of `marktree_itr_get_ext`.
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

// ============================================================================
// Lookup and Pair Functions
// ============================================================================

/// Lookup a mark by its combined lookup ID.
///
/// Returns the mark with absolute position, or an invalid key if not found.
/// If `itr` is not null, positions the iterator at the found mark.
#[must_use]
pub fn marktree_lookup(b: MarkTreeHandle, id: u64, itr: MarkTreeIterHandle) -> MTKey {
    unsafe { nvim_marktree_lookup(b, id, itr) }
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
    unsafe { nvim_marktree_lookup_ns(b, ns, id, end, itr) }
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
    unsafe { nvim_marktree_get_alt(b, mark, itr) }
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
    unsafe { nvim_marktree_get_altpos(b, mark, itr) }
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

/// Compute pseudo-index for a position in the tree.
///
/// Pseudo-indices allow efficient ordering comparisons between positions
/// without traversing the tree. They encode the path from root to the position.
#[must_use]
pub fn pseudo_index(x: MTNodeHandle, i: i32) -> u64 {
    unsafe { nvim_pseudo_index(x, i) }
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
    unsafe { nvim_pseudo_index_for_id(b, id, sloppy) }
}

/// Exported FFI version of `pseudo_index_for_id`.
#[no_mangle]
pub extern "C" fn rs_pseudo_index_for_id(b: MarkTreeHandle, id: u64, sloppy: bool) -> u64 {
    pseudo_index_for_id(b, id, sloppy)
}

/// Set iterator to point at a specific node and index.
///
/// Returns the key with absolute position.
#[must_use]
pub fn marktree_itr_set_node(
    b: MarkTreeHandle,
    itr: MarkTreeIterHandle,
    n: MTNodeHandle,
    i: i32,
) -> MTKey {
    unsafe { nvim_marktree_itr_set_node(b, itr, n, i) }
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
pub fn marktree_itr_fix_pos(b: MarkTreeHandle, itr: MarkTreeIterHandle) {
    unsafe { nvim_marktree_itr_fix_pos(b, itr) }
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
#[allow(clippy::many_single_char_names)]
#[allow(clippy::branches_sharing_code)] // Different branches have different control flow
#[must_use]
pub fn marktree_itr_next_skip(
    _b: MarkTreeHandle,
    itr: MarkTreeIterHandle,
    mut skip: bool,
    preload: bool,
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
        if !marktree_itr_next_skip(b, itr, false, false, meta_filter) {
            return false;
        }
    }
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
    if !marktree_itr_next_skip(b, itr, false, false, meta_filter) {
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
        let _ = marktree_itr_next_skip(b, itr, true, false, std::ptr::null());
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
            decor_data: 0,
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
pub fn intersect_node(b: MarkTreeHandle, x: MTNodeHandle, id: u64) {
    unsafe { nvim_intersect_node(b, x, id) }
}

/// Remove an intersection from a node.
pub fn unintersect_node(b: MarkTreeHandle, x: MTNodeHandle, id: u64, strict: bool) {
    unsafe { nvim_unintersect_node(b, x, id, strict) }
}

/// Copy intersections from one node to another.
pub fn kvi_copy_intersect(dst: MTNodeHandle, src: MTNodeHandle) {
    unsafe { nvim_kvi_copy_intersect(dst, src) }
}

/// Check if a node's intersect list contains the given ID.
#[must_use]
pub fn intersection_has(x: MTNodeHandle, id: u64) -> bool {
    unsafe { nvim_intersection_has(x, id) }
}

// ============================================================================
// B-tree Operations Wrappers
// ============================================================================

/// Split a full child node during insertion.
///
/// x must be an internal node, which is not full.
/// x->ptr[i] should be a full node, i.e. x->ptr[i]->n == 2*T-1.
pub fn split_node(b: MarkTreeHandle, x: MTNodeHandle, i: i32, next: MTKey) {
    unsafe { nvim_split_node(b, x, i, next) }
}

/// Recursive insertion helper.
///
/// x must not be a full node (even if there might be internal space).
pub fn marktree_putp_aux(
    b: MarkTreeHandle,
    x: MTNodeHandle,
    k: MTKey,
    meta_inc: &mut [u32; K_MT_META_COUNT],
) {
    unsafe { nvim_marktree_putp_aux(b, x, k, meta_inc.as_mut_ptr()) }
}

/// Insert a key into the marktree.
///
/// This is the core insertion function. It handles root splitting
/// and delegates to putp_aux for the actual insertion.
pub fn marktree_put_key(b: MarkTreeHandle, k: MTKey) {
    unsafe { nvim_marktree_put_key(b, k) }
}

/// Insert a mark with optional paired end.
///
/// If end_row >= 0, creates a paired mark with the end at (end_row, end_col).
/// The end mark will have right gravity if end_right is true.
pub fn marktree_put(b: MarkTreeHandle, key: MTKey, end_row: i32, end_col: i32, end_right: bool) {
    unsafe { nvim_marktree_put(b, key, end_row, end_col, end_right) }
}

/// Mark intersections between paired marks.
///
/// Traverses from itr to end_itr, adding (or removing if delete=true)
/// intersection markers for the paired mark identified by id.
pub fn marktree_intersect_pair(
    b: MarkTreeHandle,
    id: u64,
    itr: MarkTreeIterHandle,
    end_itr: MarkTreeIterHandle,
    delete: bool,
) {
    unsafe { nvim_marktree_intersect_pair(b, id, itr, end_itr, delete) }
}

/// Bubble up common intersections to parent.
pub fn bubble_up(x: MTNodeHandle) {
    unsafe { nvim_bubble_up(x) }
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
// Deletion Wrappers
// ============================================================================

/// Delete mark at iterator position.
///
/// Returns the ID of the paired end mark (if any), or 0 if unpaired.
/// The iterator is updated to point at the key after the deleted one.
#[must_use]
pub fn marktree_del_itr(b: MarkTreeHandle, itr: MarkTreeIterHandle, rev: bool) -> u64 {
    unsafe { nvim_marktree_del_itr(b, itr, rev) }
}

/// Revise meta counts after modifying a key's flags.
///
/// Call this after changing decoration flags on a key.
pub fn marktree_revise_meta(b: MarkTreeHandle, itr: MarkTreeIterHandle, old_key: MTKey) {
    unsafe { nvim_marktree_revise_meta(b, itr, old_key) }
}

/// Move mark to a new position.
///
/// If the new position is within the same leaf node, an optimized
/// path is taken. Otherwise, delete and re-insert.
pub fn marktree_move(b: MarkTreeHandle, itr: MarkTreeIterHandle, row: i32, col: i32) {
    unsafe { nvim_marktree_move(b, itr, row, col) }
}

/// Restore pair after move.
///
/// Re-establishes intersection markers for a paired mark.
pub fn marktree_restore_pair(b: MarkTreeHandle, key: MTKey) {
    unsafe { nvim_marktree_restore_pair(b, key) }
}

/// Pivot right (steal from left sibling).
pub fn pivot_right(b: MarkTreeHandle, p_pos: MTPos, p: MTNodeHandle, i: i32) {
    unsafe { nvim_pivot_right(b, p_pos, p, i) }
}

/// Pivot left (steal from right sibling).
pub fn pivot_left(b: MarkTreeHandle, p_pos: MTPos, p: MTNodeHandle, i: i32) {
    unsafe { nvim_pivot_left(b, p_pos, p, i) }
}

/// Merge two nodes.
#[must_use]
pub fn merge_node(b: MarkTreeHandle, p: MTNodeHandle, i: i32) -> MTNodeHandle {
    unsafe { nvim_merge_node(b, p, i) }
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

/// Free a single node.
pub fn marktree_free_node(b: MarkTreeHandle, x: MTNodeHandle) {
    unsafe { nvim_marktree_free_node(b, x) }
}

/// Free an entire subtree.
pub fn marktree_free_subtree(b: MarkTreeHandle, x: MTNodeHandle) {
    unsafe { nvim_marktree_free_subtree(b, x) }
}

/// Clear the entire marktree.
pub fn marktree_clear(b: MarkTreeHandle) {
    unsafe { nvim_marktree_clear(b) }
}

// ============================================================================
// FFI Exports for Phase 5 & 7
// ============================================================================

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

/// Exported FFI version of `merge_node`.
#[no_mangle]
pub extern "C" fn rs_merge_node(b: MarkTreeHandle, p: MTNodeHandle, i: c_int) -> MTNodeHandle {
    merge_node(b, p, i)
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

/// Splice: handle text changes in buffer.
///
/// Updates mark positions based on text change:
/// - Marks at `start` with right gravity are moved to `new_extent`
/// - Marks in the deleted region are moved to `start`
/// - Marks after the deleted region are adjusted by delta
///
/// Returns true if any marks were moved.
#[must_use]
pub fn marktree_splice(
    b: MarkTreeHandle,
    start_line: i32,
    start_col: i32,
    old_extent_line: i32,
    old_extent_col: i32,
    new_extent_line: i32,
    new_extent_col: i32,
) -> bool {
    unsafe {
        nvim_marktree_splice(
            b,
            start_line,
            start_col,
            old_extent_line,
            old_extent_col,
            new_extent_line,
            new_extent_col,
        )
    }
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
    unsafe {
        nvim_marktree_move_region(
            b, start_row, start_col, extent_row, extent_col, new_row, new_col,
        );
    }
}

// ============================================================================
// Phase 8: Debug and Validation
// ============================================================================

/// Check marktree invariants.
///
/// Validates the B-tree structure, intersection markers, and meta counts.
/// Panics if any invariant is violated.
pub fn marktree_check(b: MarkTreeHandle) {
    unsafe { nvim_marktree_check(b) }
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
            decor_data: 0,
        };
        let k2 = MTKey {
            pos: MTPos::new(0, 5),
            ns: 0,
            id: 0,
            flags: MT_FLAG_REAL,
            decor_data: 0,
        };
        let k3 = MTKey {
            pos: MTPos::new(1, 0),
            ns: 0,
            id: 0,
            flags: MT_FLAG_REAL,
            decor_data: 0,
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
