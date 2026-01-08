//! FFI exports for native Rust marktree operations.
//!
//! This module provides C-callable wrappers around the native Rust
//! marktree implementation. These functions operate on the Rust-owned
//! `MarkTree` type rather than the C opaque handle.
//!
//! Functions are prefixed with `rs_native_` to distinguish them from
//! the opaque handle wrappers in `lib.rs`.

// Allow various casts needed for B-tree level/index operations
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
// Allow pointer casts needed for NonNull
#![allow(clippy::ptr_cast_constness)]
#![allow(clippy::ref_as_ptr)]
// FFI functions cannot be const
#![allow(clippy::missing_const_for_fn)]
// FFI functions take raw pointers
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use crate::node::{MarkTree, MarkTreeIter};
use crate::{MTKey, MTPair};

// ============================================================================
// Tree Creation/Destruction
// ============================================================================

/// Create a new empty native MarkTree.
///
/// The returned pointer must be freed with `rs_native_marktree_free`.
#[no_mangle]
pub extern "C" fn rs_native_marktree_new() -> *mut MarkTree {
    Box::into_raw(Box::new(MarkTree::new()))
}

/// Free a native MarkTree.
///
/// # Safety
/// The pointer must have been created by `rs_native_marktree_new` and
/// must not be used after this call.
#[no_mangle]
pub unsafe extern "C" fn rs_native_marktree_free(tree: *mut MarkTree) {
    if !tree.is_null() {
        drop(Box::from_raw(tree));
    }
}

// ============================================================================
// Tree Statistics
// ============================================================================

/// Get the number of keys in the tree.
#[no_mangle]
pub extern "C" fn rs_native_marktree_n_keys(tree: *const MarkTree) -> usize {
    if tree.is_null() {
        return 0;
    }
    unsafe { (*tree).n_keys }
}

/// Get the number of nodes in the tree.
#[no_mangle]
pub extern "C" fn rs_native_marktree_n_nodes(tree: *const MarkTree) -> usize {
    if tree.is_null() {
        return 0;
    }
    unsafe { (*tree).n_nodes }
}

/// Get approximate memory usage in bytes.
#[no_mangle]
pub extern "C" fn rs_native_marktree_memory_usage(tree: *const MarkTree) -> usize {
    if tree.is_null() {
        return 0;
    }
    unsafe { (*tree).memory_usage() }
}

// ============================================================================
// Insertion
// ============================================================================

/// Insert a key into the tree.
#[no_mangle]
pub extern "C" fn rs_native_marktree_put_key(tree: *mut MarkTree, key: MTKey) {
    if tree.is_null() {
        return;
    }
    unsafe { (*tree).put_key(key) }
}

// ============================================================================
// Deletion
// ============================================================================

/// Delete the key at the current iterator position.
///
/// Returns the lookup ID of the deleted key, or 0 if invalid.
#[no_mangle]
pub extern "C" fn rs_native_marktree_del_itr(
    tree: *mut MarkTree,
    itr: *mut MarkTreeIter,
    _rev: bool,
) -> u64 {
    if tree.is_null() || itr.is_null() {
        return 0;
    }
    unsafe { (*tree).del_itr(&mut *itr).unwrap_or(0) }
}

/// Clear all keys from the tree.
#[no_mangle]
pub extern "C" fn rs_native_marktree_clear(tree: *mut MarkTree) {
    if tree.is_null() {
        return;
    }
    unsafe { (*tree).clear() }
}

// ============================================================================
// Splice
// ============================================================================

/// Splice the tree for a text change.
///
/// Returns true if any marks were moved.
#[no_mangle]
pub extern "C" fn rs_native_marktree_splice(
    tree: *mut MarkTree,
    start_line: i32,
    start_col: i32,
    old_extent_line: i32,
    old_extent_col: i32,
    new_extent_line: i32,
    new_extent_col: i32,
) -> bool {
    if tree.is_null() {
        return false;
    }
    unsafe {
        (*tree).splice(
            start_line,
            start_col,
            old_extent_line,
            old_extent_col,
            new_extent_line,
            new_extent_col,
        )
    }
}

// ============================================================================
// Iterator Operations
// ============================================================================

/// Create a new iterator.
///
/// The returned pointer must be freed with `rs_native_marktree_itr_free`.
#[no_mangle]
pub extern "C" fn rs_native_marktree_itr_new() -> *mut MarkTreeIter {
    Box::into_raw(Box::new(MarkTreeIter::new()))
}

/// Free an iterator.
///
/// # Safety
/// The pointer must have been created by `rs_native_marktree_itr_new`.
#[no_mangle]
pub unsafe extern "C" fn rs_native_marktree_itr_free(itr: *mut MarkTreeIter) {
    if !itr.is_null() {
        drop(Box::from_raw(itr));
    }
}

/// Position iterator at the first key.
///
/// Returns true if the tree is non-empty.
#[no_mangle]
pub extern "C" fn rs_native_marktree_itr_first(
    tree: *const MarkTree,
    itr: *mut MarkTreeIter,
) -> bool {
    if tree.is_null() || itr.is_null() {
        return false;
    }
    unsafe { (*itr).first(&*tree) }
}

/// Position iterator at the last key.
///
/// Returns true if the tree is non-empty.
#[no_mangle]
pub extern "C" fn rs_native_marktree_itr_last(
    tree: *const MarkTree,
    itr: *mut MarkTreeIter,
) -> bool {
    if tree.is_null() || itr.is_null() {
        return false;
    }
    unsafe { (*itr).last(&*tree) }
}

/// Position iterator at or after the given position.
///
/// Returns true if a key was found.
#[no_mangle]
pub extern "C" fn rs_native_marktree_itr_get(
    tree: *const MarkTree,
    itr: *mut MarkTreeIter,
    row: i32,
    col: i32,
    right_gravity: bool,
) -> bool {
    if tree.is_null() || itr.is_null() {
        return false;
    }
    unsafe { (*itr).get(&*tree, row, col, right_gravity) }
}

/// Advance iterator to next key.
///
/// Returns true if successful.
#[no_mangle]
pub extern "C" fn rs_native_marktree_itr_next(itr: *mut MarkTreeIter) -> bool {
    if itr.is_null() {
        return false;
    }
    unsafe { (*itr).next() }
}

/// Move iterator to previous key.
///
/// Returns true if successful.
#[no_mangle]
pub extern "C" fn rs_native_marktree_itr_prev(
    _tree: *const MarkTree,
    itr: *mut MarkTreeIter,
) -> bool {
    if itr.is_null() {
        return false;
    }
    unsafe { (*itr).prev() }
}

/// Check if iterator is valid.
#[no_mangle]
pub extern "C" fn rs_native_marktree_itr_valid(itr: *const MarkTreeIter) -> bool {
    if itr.is_null() {
        return false;
    }
    unsafe { (*itr).is_valid() }
}

/// Get current key at iterator position.
///
/// Returns invalid key if iterator is not valid.
#[no_mangle]
pub extern "C" fn rs_native_marktree_itr_current(itr: *const MarkTreeIter) -> MTKey {
    if itr.is_null() {
        return MTKey::invalid();
    }
    unsafe { (*itr).current().unwrap_or_else(MTKey::invalid) }
}

// ============================================================================
// Lookup
// ============================================================================

/// Lookup a key by ID.
///
/// If `itr` is non-null, positions it at the found key.
/// Returns the key, or invalid if not found.
#[no_mangle]
pub extern "C" fn rs_native_marktree_lookup(
    tree: *const MarkTree,
    id: u64,
    itr: *mut MarkTreeIter,
) -> MTKey {
    if tree.is_null() {
        return MTKey::invalid();
    }
    unsafe {
        let itr_opt = if itr.is_null() { None } else { Some(&mut *itr) };
        (*tree).lookup(itr_opt, id).unwrap_or_else(MTKey::invalid)
    }
}

// ============================================================================
// Overlap Queries
// ============================================================================

/// Initialize iterator for overlap queries at a position.
#[no_mangle]
pub extern "C" fn rs_native_marktree_itr_get_overlap(
    tree: *const MarkTree,
    itr: *mut MarkTreeIter,
    row: i32,
    col: i32,
) -> bool {
    if tree.is_null() || itr.is_null() {
        return false;
    }
    unsafe { (*itr).get_overlap(&*tree, row, col) }
}

/// Step through overlapping pairs.
///
/// Returns true if a pair was found, with the pair written to `pair`.
#[no_mangle]
pub extern "C" fn rs_native_marktree_itr_step_overlap(
    tree: *const MarkTree,
    itr: *mut MarkTreeIter,
    pair: *mut MTPair,
) -> bool {
    if tree.is_null() || itr.is_null() || pair.is_null() {
        return false;
    }
    unsafe {
        (*itr).step_overlap(&*tree).is_some_and(|p| {
            *pair = p;
            true
        })
    }
}

// ============================================================================
// Validation
// ============================================================================

/// Validate tree invariants.
///
/// Returns true if tree is valid.
#[no_mangle]
pub extern "C" fn rs_native_marktree_check(tree: *const MarkTree) -> bool {
    if tree.is_null() {
        return true;
    }
    unsafe { (*tree).check().is_ok() }
}

// ============================================================================
// Debug
// ============================================================================

/// Print tree debug information to stderr.
#[no_mangle]
pub extern "C" fn rs_native_marktree_debug_print(tree: *const MarkTree) {
    if tree.is_null() {
        eprintln!("MarkTree: null");
        return;
    }
    unsafe { (*tree).debug_print() }
}
