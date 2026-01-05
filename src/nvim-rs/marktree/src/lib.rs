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

use std::ffi::c_int;

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

// ============================================================================
// C Accessor Functions
// ============================================================================

extern "C" {
    /// Get the number of keys in a node.
    fn nvim_mtnode_get_n(x: MTNodeHandle) -> c_int;

    /// Get the level of a node (0 for leaf).
    fn nvim_mtnode_get_level(x: MTNodeHandle) -> c_int;

    /// Get a key from a node by index.
    fn nvim_mtnode_get_key(x: MTNodeHandle, idx: c_int) -> MTKey;

    /// Get a child pointer from a node by index.
    fn nvim_mtnode_get_ptr(x: MTNodeHandle, idx: c_int) -> MTNodeHandle;

    /// Get the root node of a marktree.
    fn nvim_marktree_get_root(b: MarkTreeHandle) -> MTNodeHandle;

    /// Get the total number of keys in a marktree.
    fn nvim_marktree_get_n_keys(b: MarkTreeHandle) -> usize;
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
}
