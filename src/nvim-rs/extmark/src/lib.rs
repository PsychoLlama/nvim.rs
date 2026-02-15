//! Extended marks for plugins
//!
//! This crate provides Rust implementations of extmark handling from
//! `src/nvim/extmark.c`. Extmarks are marks that sit in a MarkTree
//! data structure which provides efficient mark insertions/lookups
//! and adjustment to text changes.
//!
//! Uses an opaque handle pattern where C pointers are treated as opaque
//! handles, with field access done through C accessor functions.

#![allow(unsafe_code)] // FFI requires unsafe
#![allow(clippy::doc_markdown)] // Allow type names without backticks
#![allow(clippy::missing_const_for_fn)] // extern "C" functions cannot be const
#![allow(clippy::not_unsafe_ptr_arg_deref)] // FFI functions take raw pointers
#![allow(clippy::fn_params_excessive_bools)] // Matching C API signatures
#![allow(clippy::too_many_arguments)] // C APIs have many arguments
#![allow(clippy::cast_sign_loss)] // Required for C interop
#![allow(clippy::cast_possible_wrap)] // Required for C interop
#![allow(clippy::similar_names)] // Matching C API naming
#![allow(clippy::missing_panics_doc)] // FFI functions don't panic
#![allow(clippy::missing_safety_doc)] // TODO: Add safety docs
#![allow(clippy::must_use_candidate)] // FFI functions called from C
#![allow(clippy::semicolon_if_nothing_returned)] // Style preference
#![allow(clippy::items_after_statements)] // Matching C code style
#![allow(clippy::cast_possible_truncation)] // Values within range in practice
#![allow(dead_code)]

pub mod helpers;

use std::ffi::{c_int, c_void};

// Re-exports from dependencies for convenience
pub use nvim_buffer::BufHandle;
pub use nvim_marktree::{
    flags::*, marktree_itr_valid, DecorInlineData, MTKey, MTPair, MTPos, MarkTreeHandle,
    MarkTreeIterHandle,
};

// ============================================================================
// Type Aliases
// ============================================================================

/// Byte count type (ptrdiff_t in C).
pub type BcountT = isize;

/// Column number type (int in C).
pub type ColnrT = c_int;

/// Line number type (int64_t in C).
pub type LinenrT = i32;

// ============================================================================
// Enums
// ============================================================================

/// Undo/redo operation mode for extmarks.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtmarkOp {
    /// Extmarks shouldn't be moved
    Noop = 0,
    /// Operation should be reversible/undoable
    Undo = 1,
    /// Operation should not be reversible
    NoUndo = 2,
    /// Operation should be undoable, but not redoable
    UndoNoRedo = 3,
}

impl ExtmarkOp {
    /// Convert from C int.
    pub const fn from_c_int(val: c_int) -> Option<Self> {
        match val {
            0 => Some(Self::Noop),
            1 => Some(Self::Undo),
            2 => Some(Self::NoUndo),
            3 => Some(Self::UndoNoRedo),
            _ => None,
        }
    }
}

/// Extmark type filter for queries.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtmarkType {
    None = 0x1,
    Sign = 0x2,
    SignHL = 0x4,
    VirtText = 0x8,
    VirtLines = 0x10,
    Highlight = 0x20,
}

impl ExtmarkType {
    /// Convert from C int.
    pub const fn from_c_int(val: c_int) -> Option<Self> {
        match val {
            0x1 => Some(Self::None),
            0x2 => Some(Self::Sign),
            0x4 => Some(Self::SignHL),
            0x8 => Some(Self::VirtText),
            0x10 => Some(Self::VirtLines),
            0x20 => Some(Self::Highlight),
            _ => None,
        }
    }
}

/// Undo object type.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UndoObjectType {
    Splice = 0,
    Move = 1,
    Update = 2,
    SavePos = 3,
    Clear = 4,
}

// ============================================================================
// Handle Types
// ============================================================================

/// Opaque handle to an Error structure.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ErrorHandle(*mut c_void);

impl ErrorHandle {
    /// Check if the handle is null.
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to an extmark_undo_vec_t.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExtmarkUndoVecHandle(*mut c_void);

impl ExtmarkUndoVecHandle {
    /// Check if the handle is null.
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to a u_header_T (undo header).
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UndoHeaderHandle(*mut c_void);

impl UndoHeaderHandle {
    /// Check if the handle is null.
    #[inline]
    #[must_use]
    pub const fn is_null(self) -> bool {
        self.0.is_null()
    }
}

/// Opaque handle to an ExtmarkInfoArray (kvec_t(MTPair)).
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExtmarkInfoArrayHandle(*mut c_void);

// ============================================================================
// Data Structures (for FFI boundary)
// ============================================================================

/// Splice operation data for undo.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ExtmarkSplice {
    pub start_row: c_int,
    pub start_col: ColnrT,
    pub old_row: c_int,
    pub old_col: ColnrT,
    pub new_row: c_int,
    pub new_col: ColnrT,
    pub start_byte: BcountT,
    pub old_byte: BcountT,
    pub new_byte: BcountT,
}

/// Move operation data for undo.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ExtmarkMove {
    pub start_row: c_int,
    pub start_col: c_int,
    pub extent_row: c_int,
    pub extent_col: c_int,
    pub new_row: c_int,
    pub new_col: c_int,
    pub start_byte: BcountT,
    pub extent_byte: BcountT,
    pub new_byte: BcountT,
}

/// Saved position data for undo.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ExtmarkSavePos {
    /// Raw mark ID from marktree.
    pub mark: u64,
    pub old_row: c_int,
    pub old_col: ColnrT,
    pub invalidated: bool,
}

/// Union for undo object data.
#[repr(C)]
#[derive(Clone, Copy)]
pub union ExtmarkUndoData {
    pub splice: ExtmarkSplice,
    pub move_: ExtmarkMove,
    pub savepos: ExtmarkSavePos,
}

/// Undo object.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ExtmarkUndoObject {
    pub type_: UndoObjectType,
    pub data: ExtmarkUndoData,
}

/// Inline decoration data.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct DecorInline {
    pub ext: bool,
    pub data: DecorInlineData,
}

// ============================================================================
// C Accessor Functions
// ============================================================================

extern "C" {
    // ========================================================================
    // Buffer accessors
    // ========================================================================

    /// Get the marktree from a buffer.
    fn nvim_buf_get_marktree(buf: BufHandle) -> MarkTreeHandle;

    /// Get the deleted_bytes2 field from a buffer.
    fn nvim_buf_get_deleted_bytes2(buf: BufHandle) -> BcountT;

    /// Set the deleted_bytes2 field on a buffer.
    fn nvim_buf_set_deleted_bytes2(buf: BufHandle, val: BcountT);

    /// Get curbuf global.
    fn nvim_get_curbuf() -> BufHandle;

    /// Get curbuf_splice_pending global.
    fn nvim_get_curbuf_splice_pending() -> c_int;

    // ========================================================================
    // Map operations
    // ========================================================================

    /// Get size of extmark namespace map.
    fn nvim_extmark_ns_map_size(buf: BufHandle) -> usize;

    /// Get namespace entry if it exists.
    fn nvim_extmark_ns_get_ref(buf: BufHandle, ns_id: u32) -> *mut u32;

    /// Get or create namespace entry (initializes to 0 if new).
    fn nvim_extmark_ns_put_ref(buf: BufHandle, ns_id: u32) -> *mut u32;

    /// Delete namespace entry.
    fn nvim_extmark_ns_del(buf: BufHandle, ns_id: u32);

    /// Destroy the extmark namespace map and reinitialize.
    fn nvim_extmark_ns_destroy(buf: BufHandle);

    // ========================================================================
    // Marktree operations (calling C functions)
    // ========================================================================

    /// Delete mark at iterator position, return paired mark ID or 0.
    fn nvim_marktree_del_itr(b: MarkTreeHandle, itr: MarkTreeIterHandle, rev: bool) -> u64;

    /// Move mark at iterator position to new location.
    fn nvim_marktree_move(b: MarkTreeHandle, itr: MarkTreeIterHandle, row: c_int, col: ColnrT);

    /// Revise metadata for mark after in-place update.
    fn nvim_marktree_revise_meta(b: MarkTreeHandle, itr: MarkTreeIterHandle, old: MTKey);

    /// Get iterator to position.
    fn nvim_marktree_itr_get(b: MarkTreeHandle, row: i32, col: i32, itr: MarkTreeIterHandle);

    /// Advance iterator to next mark.
    fn nvim_marktree_itr_next(b: MarkTreeHandle, itr: MarkTreeIterHandle);

    /// Get current mark from iterator.
    fn nvim_marktree_itr_current(itr: MarkTreeIterHandle) -> MTKey;

    /// Lookup mark by ID.
    fn nvim_marktree_lookup(b: MarkTreeHandle, id: u64, itr: MarkTreeIterHandle) -> MTKey;

    /// Lookup mark by namespace and ID.
    fn nvim_marktree_lookup_ns(
        b: MarkTreeHandle,
        ns: u32,
        id: u32,
        end: bool,
        itr: MarkTreeIterHandle,
    ) -> MTKey;

    /// Get alternate end of paired mark.
    fn nvim_marktree_get_alt(b: MarkTreeHandle, mark: MTKey, itr: MarkTreeIterHandle) -> MTKey;

    /// Get position of alternate end.
    fn nvim_marktree_get_altpos(b: MarkTreeHandle, mark: MTKey, itr: MarkTreeIterHandle) -> MTPos;

    /// Initialize overlap iteration at (row, col).
    fn nvim_marktree_itr_get_overlap(
        b: MarkTreeHandle,
        row: c_int,
        col: c_int,
        itr: MarkTreeIterHandle,
    ) -> bool;

    /// Step overlap iteration, writing the next overlapping pair.
    fn nvim_marktree_itr_step_overlap(
        b: MarkTreeHandle,
        itr: MarkTreeIterHandle,
        pair: *mut MTPair,
    ) -> bool;

    /// Initialize iterator at position with simple params (no gravity, no filter).
    fn nvim_marktree_itr_get_ext_simple(
        b: MarkTreeHandle,
        row: c_int,
        col: c_int,
        itr: MarkTreeIterHandle,
    );

    /// Clear all marks from tree.
    fn nvim_marktree_clear(b: MarkTreeHandle);

    /// Splice marks for text change.
    fn nvim_marktree_splice(
        b: MarkTreeHandle,
        start_row: i32,
        start_col: ColnrT,
        old_row: c_int,
        old_col: ColnrT,
        new_row: c_int,
        new_col: ColnrT,
    );

    /// Move region of marks.
    fn nvim_marktree_move_region(
        b: MarkTreeHandle,
        start_row: c_int,
        start_col: ColnrT,
        extent_row: c_int,
        extent_col: ColnrT,
        new_row: c_int,
        new_col: ColnrT,
    );

    // ========================================================================
    // Iterator allocation (C-side)
    // ========================================================================

    /// Allocate a new MarkTreeIter on the heap.
    fn nvim_marktree_itr_alloc() -> MarkTreeIterHandle;

    /// Free a heap-allocated MarkTreeIter.
    fn nvim_marktree_itr_free(itr: MarkTreeIterHandle);

    /// Copy iterator contents.
    fn nvim_marktree_itr_copy(dst: MarkTreeIterHandle, src: MarkTreeIterHandle);

    // ========================================================================
    // Iterator rawkey access (for in-place modification)
    // ========================================================================

    /// Get flags from mark at iterator position (raw access).
    fn nvim_mt_itr_rawkey_get_flags(itr: MarkTreeIterHandle) -> u16;

    /// Set flags on mark at iterator position (raw access).
    fn nvim_mt_itr_rawkey_set_flags(itr: MarkTreeIterHandle, flags: u16);

    /// Get decor_data from mark at iterator position (raw access).
    fn nvim_mt_itr_rawkey_get_decor_data(itr: MarkTreeIterHandle) -> DecorInlineData;

    /// Set decor_data on mark at iterator position (raw access).
    fn nvim_mt_itr_rawkey_set_decor_data(itr: MarkTreeIterHandle, data: DecorInlineData);

    // ========================================================================
    // Marktree put (insert mark)
    // ========================================================================

    /// Insert a mark into the marktree with optional end position.
    fn nvim_marktree_put(
        b: MarkTreeHandle,
        key: MTKey,
        end_row: c_int,
        end_col: c_int,
        end_right: bool,
    );

    // ========================================================================
    // Decoration operations
    // ========================================================================

    /// Free decoration data.
    fn nvim_decor_free(decor: DecorInlineData, ext: bool);

    /// Remove decoration from buffer.
    fn nvim_buf_decor_remove(
        buf: BufHandle,
        row1: c_int,
        row2: c_int,
        col: ColnrT,
        decor_data: DecorInlineData,
        decor_ext: bool,
        free: bool,
    );

    /// Add decoration to buffer.
    fn nvim_buf_put_decor(
        buf: BufHandle,
        decor_data: DecorInlineData,
        decor_ext: bool,
        row1: c_int,
        row2: c_int,
    );

    /// Redraw decoration.
    fn nvim_decor_redraw(
        buf: BufHandle,
        row1: c_int,
        row2: c_int,
        col: ColnrT,
        decor_data: DecorInlineData,
        decor_ext: bool,
    );

    /// Invalidate decoration state.
    fn nvim_decor_state_invalidate(buf: BufHandle);

    /// Get type flags for decoration data.
    fn nvim_decor_type_flags(data: DecorInlineData, ext: bool) -> u16;

    // ========================================================================
    // ExtmarkInfoArray operations
    // ========================================================================

    /// Get the size of an ExtmarkInfoArray.
    fn nvim_extmark_array_size(array: ExtmarkInfoArrayHandle) -> i64;

    /// Push an MTPair onto an ExtmarkInfoArray.
    fn nvim_extmark_array_push(array: ExtmarkInfoArrayHandle, pair: MTPair);

    // ========================================================================
    // Buffer updates
    // ========================================================================

    /// Send splice event to buffer update listeners.
    fn nvim_buf_updates_send_splice(
        buf: BufHandle,
        start_row: c_int,
        start_col: ColnrT,
        start_byte: BcountT,
        old_row: c_int,
        old_col: ColnrT,
        old_byte: BcountT,
        new_row: c_int,
        new_col: ColnrT,
        new_byte: BcountT,
    );

    // ========================================================================
    // Sign column operations
    // ========================================================================

    /// Get autom field from buffer signcols.
    fn nvim_buf_signcols_get_autom(buf: BufHandle) -> bool;

    /// Count signs in range.
    fn nvim_buf_signcols_count_range(
        buf: BufHandle,
        row1: c_int,
        row2: c_int,
        add: c_int,
        clear: c_int,
    );

    /// Clear buffer signcols max and count.
    fn nvim_buf_signcols_clear(buf: BufHandle);

    /// Get ml_line_count from buffer.
    fn nvim_buf_get_ml_line_count(buf: BufHandle) -> LinenrT;

    /// Get prev_line_count from buffer.
    fn nvim_buf_get_prev_line_count(buf: BufHandle) -> c_int;

    /// Set prev_line_count on buffer.
    fn nvim_buf_set_prev_line_count(buf: BufHandle, val: c_int);

    // ========================================================================
    // Memline operations
    // ========================================================================

    /// Find line offset.
    fn nvim_ml_find_line_or_offset(
        buf: BufHandle,
        lnum: LinenrT,
        offp: *mut BcountT,
        no_ff: bool,
    ) -> BcountT;

    // ========================================================================
    // Undo operations
    // ========================================================================

    /// Force get undo header.
    fn nvim_u_force_get_undo_header(buf: BufHandle) -> UndoHeaderHandle;

    /// Get extmark undo vec from undo header.
    fn nvim_uhp_get_extmark(uhp: UndoHeaderHandle) -> ExtmarkUndoVecHandle;

    /// Get size of extmark undo vec.
    fn nvim_extmark_undo_vec_size(uvp: ExtmarkUndoVecHandle) -> usize;

    /// Push element to extmark undo vec.
    fn nvim_extmark_undo_vec_push(uvp: ExtmarkUndoVecHandle, obj: ExtmarkUndoObject);

    /// Get last element from extmark undo vec.
    fn nvim_extmark_undo_vec_last(uvp: ExtmarkUndoVecHandle) -> *mut ExtmarkUndoObject;
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Compute flags for a new mark.
/// Note: Does NOT set MT_FLAG_REAL - that's in MT_FLAG_EXTERNAL_MASK and
/// gets added separately via decor_flags in extmark_set.
#[inline]
#[must_use]
pub fn mt_flags(right_gravity: bool, no_undo: bool, invalidate: bool, decor_ext: bool) -> u16 {
    let mut flags = 0u16;
    if right_gravity {
        flags |= MT_FLAG_RIGHT_GRAVITY;
    }
    if no_undo {
        flags |= MT_FLAG_NO_UNDO;
    }
    if invalidate {
        flags |= MT_FLAG_INVALIDATE;
    }
    if decor_ext {
        flags |= MT_FLAG_DECOR_EXT;
    }
    flags
}

/// Check if mark is paired.
#[inline]
#[must_use]
pub const fn mt_paired(key: MTKey) -> bool {
    key.flags & MT_FLAG_PAIRED != 0
}

/// Check if mark is the end of a pair.
#[inline]
#[must_use]
pub const fn mt_end(key: MTKey) -> bool {
    key.flags & MT_FLAG_END != 0
}

/// Check if mark has right gravity.
#[inline]
#[must_use]
pub const fn mt_right(key: MTKey) -> bool {
    key.flags & MT_FLAG_RIGHT_GRAVITY != 0
}

/// Check if mark is invalid.
#[inline]
#[must_use]
pub const fn mt_invalid(key: MTKey) -> bool {
    key.flags & MT_FLAG_INVALID != 0
}

/// Check if mark should be invalidated.
#[inline]
#[must_use]
pub const fn mt_invalidate(key: MTKey) -> bool {
    key.flags & MT_FLAG_INVALIDATE != 0
}

/// Check if mark has no_undo flag.
#[inline]
#[must_use]
pub const fn mt_no_undo(key: MTKey) -> bool {
    key.flags & MT_FLAG_NO_UNDO != 0
}

/// Check if mark has any decoration.
#[inline]
#[must_use]
pub const fn mt_decor_any(key: MTKey) -> bool {
    key.flags & MT_FLAG_DECOR_MASK != 0
}

/// Check if mark has external decoration.
#[inline]
#[must_use]
pub const fn mt_decor_ext(key: MTKey) -> bool {
    key.flags & MT_FLAG_DECOR_EXT != 0
}

/// Get decoration data as (data, ext) pair.
#[inline]
#[must_use]
pub const fn mt_decor(key: MTKey) -> (DecorInlineData, bool) {
    (key.decor_data, key.flags & MT_FLAG_DECOR_EXT != 0)
}

/// Convert mark to lookup key.
#[inline]
#[must_use]
pub const fn mt_lookup_key(key: MTKey) -> u64 {
    // Lookup key encodes ns, id, and end flag
    ((key.ns as u64) << 32) | (key.id as u64) | (if mt_end(key) { 1 } else { 0 })
}

// ============================================================================
// Utility functions
// ============================================================================

/// Get the current buffer.
#[inline]
fn get_curbuf() -> BufHandle {
    unsafe { nvim_get_curbuf() }
}

// ============================================================================
// Core Extmark Operations
// ============================================================================

/// Create or update an extmark.
///
/// Handles namespace ID tracking, in-place revision of existing marks,
/// decoration registration, and mark insertion into the marktree.
#[export_name = "extmark_set"]
#[allow(clippy::too_many_arguments)]
pub extern "C" fn rs_extmark_set(
    buf: BufHandle,
    ns_id: u32,
    idp: *mut u32,
    row: c_int,
    col: ColnrT,
    end_row: c_int,
    end_col: ColnrT,
    decor: DecorInline,
    decor_flags: u16,
    right_gravity: bool,
    end_right_gravity: bool,
    no_undo: bool,
    invalidate: bool,
    _err: ErrorHandle,
) {
    let ns = unsafe { nvim_extmark_ns_put_ref(buf, ns_id) };
    let mut id = if idp.is_null() { 0 } else { unsafe { *idp } };

    let flags = mt_flags(right_gravity, no_undo, invalidate, decor.ext) | decor_flags;
    let mut revised = false;

    if id == 0 {
        unsafe {
            *ns += 1;
            id = *ns;
        }
    } else {
        let itr = unsafe { nvim_marktree_itr_alloc() };
        let tree = unsafe { nvim_buf_get_marktree(buf) };
        let old_mark = unsafe { nvim_marktree_lookup_ns(tree, ns_id, id, false, itr) };

        if old_mark.id != 0 {
            if mt_paired(old_mark) || end_row > -1 {
                extmark_del_id(buf, ns_id, id);
            } else {
                assert!(marktree_itr_valid(itr));
                if old_mark.pos.row == row && old_mark.pos.col == col {
                    // not paired: we can revise in place
                    if !mt_invalid(old_mark) && mt_decor_any(old_mark) {
                        let cur_flags = unsafe { nvim_mt_itr_rawkey_get_flags(itr) };
                        unsafe {
                            nvim_mt_itr_rawkey_set_flags(itr, cur_flags & !MT_FLAG_EXTERNAL_MASK)
                        };
                        let (decor_data, ext) = mt_decor(old_mark);
                        unsafe { nvim_buf_decor_remove(buf, row, row, col, decor_data, ext, true) };
                    }
                    let cur_flags = unsafe { nvim_mt_itr_rawkey_get_flags(itr) };
                    unsafe { nvim_mt_itr_rawkey_set_flags(itr, cur_flags | flags) };
                    unsafe { nvim_mt_itr_rawkey_set_decor_data(itr, decor.data) };
                    unsafe { nvim_marktree_revise_meta(tree, itr, old_mark) };
                    revised = true;
                } else {
                    unsafe { nvim_marktree_del_itr(tree, itr, false) };
                    if !mt_invalid(old_mark) {
                        let (decor_data, ext) = mt_decor(old_mark);
                        unsafe {
                            nvim_buf_decor_remove(
                                buf,
                                old_mark.pos.row,
                                old_mark.pos.row,
                                old_mark.pos.col,
                                decor_data,
                                ext,
                                true,
                            )
                        };
                    }
                }
            }
        } else {
            unsafe {
                *ns = (*ns).max(id);
            }
        }

        unsafe { nvim_marktree_itr_free(itr) };
    }

    if !revised {
        let mark = MTKey {
            pos: MTPos { row, col },
            ns: ns_id,
            id,
            flags,
            decor_data: decor.data,
        };
        let tree = unsafe { nvim_buf_get_marktree(buf) };
        unsafe { nvim_marktree_put(tree, mark, end_row, end_col, end_right_gravity) };
        unsafe { nvim_decor_state_invalidate(buf) };
    }

    if decor_flags != 0 || decor.ext {
        let end = if end_row > -1 { end_row } else { row };
        unsafe { nvim_buf_put_decor(buf, decor.data, decor.ext, row, end) };
        unsafe { nvim_decor_redraw(buf, row, end, col, decor.data, decor.ext) };
    }

    if !idp.is_null() {
        unsafe { *idp = id };
    }
}

/// Remove an extmark by namespace and ID.
///
/// Returns true if the mark was found and deleted.
#[export_name = "extmark_del_id"]
pub extern "C" fn rs_extmark_del_id(buf: BufHandle, ns_id: u32, id: u32) -> bool {
    extmark_del_id(buf, ns_id, id)
}

/// Remove an extmark by namespace and ID (internal implementation).
pub fn extmark_del_id(buf: BufHandle, ns_id: u32, id: u32) -> bool {
    let itr = unsafe { nvim_marktree_itr_alloc() };
    let tree = unsafe { nvim_buf_get_marktree(buf) };
    let key = unsafe { nvim_marktree_lookup_ns(tree, ns_id, id, false, itr) };

    let result = if key.id != 0 {
        extmark_del(buf, itr, key, false);
        true
    } else {
        false
    };

    unsafe { nvim_marktree_itr_free(itr) };
    result
}

/// Remove a (paired) extmark pointed to by iterator.
#[export_name = "extmark_del"]
pub extern "C" fn rs_extmark_del(
    buf: BufHandle,
    itr: MarkTreeIterHandle,
    key: MTKey,
    restore: bool,
) {
    extmark_del(buf, itr, key, restore);
}

/// Remove a (paired) extmark pointed to by iterator (internal implementation).
pub fn extmark_del(buf: BufHandle, itr: MarkTreeIterHandle, key: MTKey, restore: bool) {
    assert!(key.pos.row >= 0);

    let tree = unsafe { nvim_buf_get_marktree(buf) };
    let mut key2 = key;
    let other = unsafe { nvim_marktree_del_itr(tree, itr, false) };

    if other != 0 {
        let alt_itr = unsafe { nvim_marktree_itr_alloc() };
        key2 = unsafe { nvim_marktree_lookup(tree, other, alt_itr) };
        assert!(key2.pos.row >= 0);
        unsafe { nvim_marktree_del_itr(tree, alt_itr, false) };
        if restore {
            unsafe { nvim_marktree_itr_get(tree, key.pos.row, key.pos.col, itr) };
        }
        unsafe { nvim_marktree_itr_free(alt_itr) };
    }

    if mt_decor_any(key) {
        let (decor_data, ext) = mt_decor(key);
        if mt_invalid(key) {
            unsafe { nvim_decor_free(decor_data, ext) };
        } else {
            let (k, k2) = if mt_end(key) {
                (key2, key)
            } else {
                (key, key2)
            };
            unsafe {
                nvim_buf_decor_remove(buf, k.pos.row, k2.pos.row, k.pos.col, decor_data, ext, true)
            };
        }
    }

    unsafe { nvim_decor_state_invalidate(buf) };
}

/// Clear extmarks in a namespace between lines.
///
/// If ns_id is 0, clears all namespaces.
#[export_name = "extmark_clear"]
pub extern "C" fn rs_extmark_clear(
    buf: BufHandle,
    ns_id: u32,
    l_row: c_int,
    l_col: ColnrT,
    u_row: c_int,
    u_col: ColnrT,
) -> bool {
    extmark_clear(buf, ns_id, l_row, l_col, u_row, u_col)
}

/// Clear extmarks in a namespace between lines (internal implementation).
pub fn extmark_clear(
    buf: BufHandle,
    ns_id: u32,
    l_row: c_int,
    l_col: ColnrT,
    u_row: c_int,
    u_col: ColnrT,
) -> bool {
    if unsafe { nvim_extmark_ns_map_size(buf) } == 0 {
        return false;
    }

    let all_ns = ns_id == 0;
    if !all_ns {
        let ns = unsafe { nvim_extmark_ns_get_ref(buf, ns_id) };
        if ns.is_null() {
            return false;
        }
    }

    let mut marks_cleared_any = false;
    let mut marks_cleared_all = l_row == 0 && l_col == 0;

    let itr = unsafe { nvim_marktree_itr_alloc() };
    let tree = unsafe { nvim_buf_get_marktree(buf) };
    unsafe { nvim_marktree_itr_get(tree, l_row, l_col, itr) };

    loop {
        let mark = unsafe { nvim_marktree_itr_current(itr) };
        if mark.pos.row < 0
            || mark.pos.row > u_row
            || (mark.pos.row == u_row && mark.pos.col > u_col)
        {
            if mark.pos.row >= 0 {
                marks_cleared_all = false;
            }
            break;
        }
        if mark.ns == ns_id || all_ns {
            marks_cleared_any = true;
            extmark_del(buf, itr, mark, true);
        } else {
            unsafe { nvim_marktree_itr_next(tree, itr) };
        }
    }

    unsafe { nvim_marktree_itr_free(itr) };

    if marks_cleared_all {
        if all_ns {
            unsafe { nvim_extmark_ns_destroy(buf) };
        } else {
            unsafe { nvim_extmark_ns_del(buf, ns_id) };
        }
    }

    if marks_cleared_any {
        unsafe { nvim_decor_state_invalidate(buf) };
    }

    marks_cleared_any
}

/// Filter and push a mark pair into the result array.
///
/// Filters by namespace and type before pushing.
fn push_mark(array: ExtmarkInfoArrayHandle, ns_id: u32, type_filter: c_int, mark: MTPair) {
    // Filter by namespace: UINT32_MAX means all namespaces
    if !(ns_id == u32::MAX || mark.start.ns == ns_id) {
        return;
    }

    // Filter by type
    let k_extmark_none: c_int = 0x1;
    if type_filter != k_extmark_none {
        if !mt_decor_any(mark.start) {
            return;
        }
        let (decor_data, ext) = mt_decor(mark.start);
        let type_flags = unsafe { nvim_decor_type_flags(decor_data, ext) };
        if c_int::from(type_flags) & type_filter == 0 {
            return;
        }
    }

    unsafe { nvim_extmark_array_push(array, mark) };
}

/// Get extmarks in a range, populating the given array.
///
/// Implements the overlap and regular iteration paths, filtering by namespace
/// and decoration type.
#[no_mangle]
pub extern "C" fn rs_extmark_get(
    buf: BufHandle,
    ns_id: u32,
    l_row: c_int,
    l_col: ColnrT,
    u_row: c_int,
    u_col: ColnrT,
    amount: i64,
    type_filter: c_int,
    overlap: bool,
    array: ExtmarkInfoArrayHandle,
) {
    let tree = unsafe { nvim_buf_get_marktree(buf) };
    let itr = unsafe { nvim_marktree_itr_alloc() };

    if overlap {
        // Find all marks overlapping the start position
        if !unsafe { nvim_marktree_itr_get_overlap(tree, l_row, l_col, itr) } {
            unsafe { nvim_marktree_itr_free(itr) };
            return;
        }

        let mut pair = MTPair::zero();
        while unsafe { nvim_marktree_itr_step_overlap(tree, itr, &raw mut pair) } {
            push_mark(array, ns_id, type_filter, pair);
        }
    } else {
        // Find all marks beginning at the start position
        unsafe { nvim_marktree_itr_get_ext_simple(tree, l_row, l_col, itr) };
    }

    while unsafe { nvim_extmark_array_size(array) } < amount {
        let mark = unsafe { nvim_marktree_itr_current(itr) };
        if mark.pos.row < 0
            || (mark.pos.row > u_row || (mark.pos.row == u_row && mark.pos.col > u_col))
        {
            break;
        }
        if !mt_end(mark) {
            let end = unsafe { nvim_marktree_get_alt(tree, mark, MarkTreeIterHandle::null()) };
            push_mark(array, ns_id, type_filter, MTPair::from_keys(mark, end));
        }
        unsafe { nvim_marktree_itr_next(tree, itr) };
    }

    unsafe { nvim_marktree_itr_free(itr) };
}

/// Lookup an extmark by ID and return the mark pair.
#[export_name = "extmark_from_id"]
pub extern "C" fn rs_extmark_from_id(buf: BufHandle, ns_id: u32, id: u32) -> MTPair {
    extmark_from_id(buf, ns_id, id)
}

/// Lookup an extmark by ID (internal implementation).
pub fn extmark_from_id(buf: BufHandle, ns_id: u32, id: u32) -> MTPair {
    let tree = unsafe { nvim_buf_get_marktree(buf) };
    let mark =
        unsafe { nvim_marktree_lookup_ns(tree, ns_id, id, false, MarkTreeIterHandle::null()) };

    if mark.id == 0 {
        return MTPair::from_keys(mark, mark);
    }

    assert!(mark.pos.row >= 0);
    let end = unsafe { nvim_marktree_get_alt(tree, mark, MarkTreeIterHandle::null()) };

    MTPair::from_keys(mark, end)
}

/// Free all extmarks from a buffer.
#[export_name = "extmark_free_all"]
pub extern "C" fn rs_extmark_free_all(buf: BufHandle) {
    extmark_free_all(buf);
}

/// Free all extmarks from a buffer (internal implementation).
pub fn extmark_free_all(buf: BufHandle) {
    let itr = unsafe { nvim_marktree_itr_alloc() };
    let tree = unsafe { nvim_buf_get_marktree(buf) };
    unsafe { nvim_marktree_itr_get(tree, 0, 0, itr) };

    loop {
        let mark = unsafe { nvim_marktree_itr_current(itr) };
        if mark.pos.row < 0 {
            break;
        }

        // Don't free mark.decor twice for a paired mark
        if !(mt_paired(mark) && mt_end(mark)) {
            let (decor_data, ext) = mt_decor(mark);
            unsafe { nvim_decor_free(decor_data, ext) };
        }

        unsafe { nvim_marktree_itr_next(tree, itr) };
    }

    unsafe { nvim_marktree_itr_free(itr) };

    unsafe { nvim_marktree_clear(tree) };
    unsafe { nvim_buf_signcols_clear(buf) };
    unsafe { nvim_extmark_ns_destroy(buf) };
}

/// Adjust extmark row for inserted/deleted rows.
#[export_name = "extmark_adjust"]
pub extern "C" fn rs_extmark_adjust(
    buf: BufHandle,
    line1: LinenrT,
    line2: LinenrT,
    amount: LinenrT,
    amount_after: LinenrT,
    undo: c_int,
) {
    let undo_op = ExtmarkOp::from_c_int(undo).unwrap_or(ExtmarkOp::Noop);
    extmark_adjust(buf, line1, line2, amount, amount_after, undo_op);
}

/// Adjust extmark row for inserted/deleted rows (internal implementation).
pub fn extmark_adjust(
    buf: BufHandle,
    line1: LinenrT,
    line2: LinenrT,
    amount: LinenrT,
    amount_after: LinenrT,
    undo: ExtmarkOp,
) {
    if unsafe { nvim_get_curbuf_splice_pending() } != 0 {
        return;
    }

    let start_byte = unsafe { nvim_ml_find_line_or_offset(buf, line1, std::ptr::null_mut(), true) };
    let mut old_byte: BcountT = 0;
    let mut new_byte: BcountT = 0;
    let old_row: c_int;
    let new_row: c_int;

    const MAXLNUM: LinenrT = 0x7fff_ffff;

    if amount == MAXLNUM {
        old_row = (line2 - line1 + 1) as c_int;
        old_byte = unsafe { nvim_buf_get_deleted_bytes2(buf) };
        new_row = amount_after as c_int + old_row;
    } else {
        assert!(line2 == MAXLNUM);
        old_row = 0;
        new_row = amount as c_int;
    }

    if new_row > 0 {
        new_byte = unsafe {
            nvim_ml_find_line_or_offset(buf, line1 + new_row as LinenrT, std::ptr::null_mut(), true)
        } - start_byte;
    }

    extmark_splice_impl(
        buf,
        line1 as c_int - 1,
        0,
        start_byte,
        old_row,
        0,
        old_byte,
        new_row,
        0,
        new_byte,
        undo,
    );
}

/// Splice extmarks for a text change.
#[export_name = "extmark_splice"]
pub extern "C" fn rs_extmark_splice(
    buf: BufHandle,
    start_row: c_int,
    start_col: ColnrT,
    old_row: c_int,
    old_col: ColnrT,
    old_byte: BcountT,
    new_row: c_int,
    new_col: ColnrT,
    new_byte: BcountT,
    undo: c_int,
) {
    let undo_op = ExtmarkOp::from_c_int(undo).unwrap_or(ExtmarkOp::Noop);
    extmark_splice(
        buf, start_row, start_col, old_row, old_col, old_byte, new_row, new_col, new_byte, undo_op,
    );
}

/// Splice extmarks for a text change (internal implementation).
pub fn extmark_splice(
    buf: BufHandle,
    start_row: c_int,
    start_col: ColnrT,
    old_row: c_int,
    old_col: ColnrT,
    old_byte: BcountT,
    new_row: c_int,
    new_col: ColnrT,
    new_byte: BcountT,
    undo: ExtmarkOp,
) {
    let mut offset =
        unsafe { nvim_ml_find_line_or_offset(buf, start_row + 1, std::ptr::null_mut(), true) };

    // On empty buffers, when editing the first line, the line is buffered,
    // causing offset to be < 0.
    if offset < 0 {
        offset = 0;
    }

    extmark_splice_impl(
        buf,
        start_row,
        start_col,
        offset + start_col as BcountT,
        old_row,
        old_col,
        old_byte,
        new_row,
        new_col,
        new_byte,
        undo,
    );
}

/// Splice extmarks (column-only change).
#[export_name = "extmark_splice_cols"]
pub extern "C" fn rs_extmark_splice_cols(
    buf: BufHandle,
    start_row: c_int,
    start_col: ColnrT,
    old_col: ColnrT,
    new_col: ColnrT,
    undo: c_int,
) {
    let undo_op = ExtmarkOp::from_c_int(undo).unwrap_or(ExtmarkOp::Noop);
    extmark_splice(
        buf,
        start_row,
        start_col,
        0,
        old_col,
        old_col as BcountT,
        0,
        new_col,
        new_col as BcountT,
        undo_op,
    );
}

/// Internal implementation of extmark splice (FFI export).
#[export_name = "extmark_splice_impl"]
pub extern "C" fn rs_extmark_splice_impl(
    buf: BufHandle,
    start_row: c_int,
    start_col: ColnrT,
    start_byte: BcountT,
    old_row: c_int,
    old_col: ColnrT,
    old_byte: BcountT,
    new_row: c_int,
    new_col: ColnrT,
    new_byte: BcountT,
    undo: c_int,
) {
    let undo_op = ExtmarkOp::from_c_int(undo).unwrap_or(ExtmarkOp::Noop);
    extmark_splice_impl(
        buf, start_row, start_col, start_byte, old_row, old_col, old_byte, new_row, new_col,
        new_byte, undo_op,
    );
}

/// Internal implementation of extmark splice.
pub fn extmark_splice_impl(
    buf: BufHandle,
    start_row: c_int,
    start_col: ColnrT,
    start_byte: BcountT,
    old_row: c_int,
    old_col: ColnrT,
    old_byte: BcountT,
    new_row: c_int,
    new_col: ColnrT,
    new_byte: BcountT,
    undo: ExtmarkOp,
) {
    unsafe { nvim_buf_set_deleted_bytes2(buf, 0) };
    unsafe {
        nvim_buf_updates_send_splice(
            buf, start_row, start_col, start_byte, old_row, old_col, old_byte, new_row, new_col,
            new_byte,
        )
    };

    if old_row > 0 || old_col > 0 {
        let end_row = start_row + old_row;
        let end_col = if old_row != 0 { 0 } else { start_col } + old_col;
        let uhp = unsafe { nvim_u_force_get_undo_header(buf) };
        let uvp = if uhp.is_null() {
            ExtmarkUndoVecHandle(std::ptr::null_mut())
        } else {
            unsafe { nvim_uhp_get_extmark(uhp) }
        };
        extmark_splice_delete(
            buf, start_row, start_col, end_row, end_col, uvp, false, undo,
        );
    }

    // Remove signs inside edited region from "b_signcols.count", add after splicing
    if old_row > 0 || new_row > 0 {
        let prev_count = unsafe { nvim_buf_get_prev_line_count(buf) };
        let count = if prev_count > 0 {
            prev_count
        } else {
            unsafe { nvim_buf_get_ml_line_count(buf) as c_int }
        };
        let row2 = (count - 1).min(start_row + old_row);
        unsafe { nvim_buf_signcols_count_range(buf, start_row, row2, 0, 1) }; // kTrue = 1
        unsafe { nvim_buf_set_prev_line_count(buf, 0) };
    }

    let tree = unsafe { nvim_buf_get_marktree(buf) };
    unsafe {
        nvim_marktree_splice(
            tree, start_row, start_col, old_row, old_col, new_row, new_col,
        )
    };

    if old_row > 0 || new_row > 0 {
        let line_count = unsafe { nvim_buf_get_ml_line_count(buf) as c_int };
        let row2 = (line_count - 1).min(start_row + new_row);
        unsafe { nvim_buf_signcols_count_range(buf, start_row, row2, 0, 0) }; // kNone = 0
    }

    if undo == ExtmarkOp::Undo {
        let uhp = unsafe { nvim_u_force_get_undo_header(buf) };
        if uhp.is_null() {
            return;
        }

        let uvp = unsafe { nvim_uhp_get_extmark(uhp) };
        let uvp_size = unsafe { nvim_extmark_undo_vec_size(uvp) };

        let mut merged = false;

        // Try to merge with last undo entry
        if old_row == 0 && new_row == 0 && uvp_size > 0 {
            let last = unsafe { nvim_extmark_undo_vec_last(uvp) };
            if !last.is_null() {
                let item = unsafe { &mut *last };
                if matches!(item.type_, UndoObjectType::Splice) {
                    let splice = unsafe { &mut item.data.splice };
                    if splice.start_row == start_row && splice.old_row == 0 && splice.new_row == 0 {
                        if old_col == 0
                            && start_col >= splice.start_col
                            && start_col <= splice.start_col + splice.new_col
                        {
                            splice.new_col += new_col;
                            splice.new_byte += new_byte;
                            merged = true;
                        } else if new_col == 0 && start_col == splice.start_col + splice.new_col {
                            splice.old_col += old_col;
                            splice.old_byte += old_byte;
                            merged = true;
                        } else if new_col == 0 && start_col + old_col == splice.start_col {
                            splice.start_col = start_col;
                            splice.start_byte = start_byte;
                            splice.old_col += old_col;
                            splice.old_byte += old_byte;
                            merged = true;
                        }
                    }
                }
            }
        }

        if !merged {
            let splice = ExtmarkSplice {
                start_row,
                start_col,
                old_row,
                old_col,
                new_row,
                new_col,
                start_byte,
                old_byte,
                new_byte,
            };

            let obj = ExtmarkUndoObject {
                type_: UndoObjectType::Splice,
                data: ExtmarkUndoData { splice },
            };

            unsafe { nvim_extmark_undo_vec_push(uvp, obj) };
        }
    }
}

/// Invalidate extmarks between range and copy to undo header (FFI export).
#[export_name = "extmark_splice_delete"]
pub extern "C" fn rs_extmark_splice_delete(
    buf: BufHandle,
    l_row: c_int,
    l_col: ColnrT,
    u_row: c_int,
    u_col: ColnrT,
    uvp: ExtmarkUndoVecHandle,
    only_copy: bool,
    op: c_int,
) {
    let op = ExtmarkOp::from_c_int(op).unwrap_or(ExtmarkOp::Noop);
    extmark_splice_delete(buf, l_row, l_col, u_row, u_col, uvp, only_copy, op);
}

/// Invalidate extmarks between range and copy to undo header.
pub fn extmark_splice_delete(
    buf: BufHandle,
    l_row: c_int,
    l_col: ColnrT,
    u_row: c_int,
    u_col: ColnrT,
    uvp: ExtmarkUndoVecHandle,
    only_copy: bool,
    op: ExtmarkOp,
) {
    let itr = unsafe { nvim_marktree_itr_alloc() };
    let tree = unsafe { nvim_buf_get_marktree(buf) };
    unsafe { nvim_marktree_itr_get(tree, l_row, l_col, itr) };

    loop {
        let mark = unsafe { nvim_marktree_itr_current(itr) };
        if mark.pos.row < 0 || mark.pos.row > u_row {
            break;
        }

        let mut copy = true;
        // No need to copy left gravity marks at the beginning of the range,
        // and right gravity marks at the end of the range, unless invalidated.
        if mark.pos.row == l_row && mark.pos.col - c_int::from(!mt_right(mark)) < l_col {
            copy = false;
        } else if mark.pos.row == u_row {
            if mark.pos.col > u_col + 1 {
                break;
            } else if mark.pos.col + c_int::from(mt_right(mark)) > u_col {
                copy = false;
            }
        }

        let mut invalidated = false;

        // Invalidate/delete mark
        if !only_copy && !mt_invalid(mark) && mt_invalidate(mark) && !mt_end(mark) {
            let end_itr = unsafe { nvim_marktree_itr_alloc() };
            unsafe { nvim_marktree_itr_copy(end_itr, itr) };

            let endpos = unsafe { nvim_marktree_get_altpos(tree, mark, end_itr) };

            // Determine if mark should be invalidated
            let should_invalidate = if !mt_paired(mark) && mark.pos.row < u_row {
                true
            } else if mt_paired(mark) {
                let u_row_adj = u_row - i32::from(u_col == 0);
                (endpos.col <= u_col || (u_col == 0 && endpos.row == mark.pos.row))
                    && mark.pos.col >= l_col
                    && mark.pos.row >= l_row
                    && endpos.row <= u_row_adj
            } else {
                false
            };

            if should_invalidate {
                if mt_no_undo(mark) {
                    unsafe { nvim_marktree_itr_free(end_itr) };
                    extmark_del(buf, itr, mark, true);
                    continue;
                }
                copy = true;
                invalidated = true;
                let flags = unsafe { nvim_mt_itr_rawkey_get_flags(itr) };
                unsafe { nvim_mt_itr_rawkey_set_flags(itr, flags | MT_FLAG_INVALID) };
                let end_flags = unsafe { nvim_mt_itr_rawkey_get_flags(end_itr) };
                unsafe { nvim_mt_itr_rawkey_set_flags(end_itr, end_flags | MT_FLAG_INVALID) };
                unsafe { nvim_marktree_revise_meta(tree, itr, mark) };
                let (decor_data, ext) = mt_decor(mark);
                unsafe {
                    nvim_buf_decor_remove(
                        buf,
                        mark.pos.row,
                        endpos.row,
                        mark.pos.col,
                        decor_data,
                        ext,
                        false,
                    )
                };
            }

            unsafe { nvim_marktree_itr_free(end_itr) };
        }

        // Push mark to undo header
        if copy && (only_copy || (!uvp.is_null() && op == ExtmarkOp::Undo && !mt_no_undo(mark))) {
            let savepos = ExtmarkSavePos {
                mark: mt_lookup_key(mark),
                invalidated,
                old_row: mark.pos.row,
                old_col: mark.pos.col,
            };

            let obj = ExtmarkUndoObject {
                type_: UndoObjectType::SavePos,
                data: ExtmarkUndoData { savepos },
            };

            unsafe { nvim_extmark_undo_vec_push(uvp, obj) };
        }

        unsafe { nvim_marktree_itr_next(tree, itr) };
    }

    unsafe { nvim_marktree_itr_free(itr) };
}

/// Apply undo or redo for an extmark operation.
#[export_name = "extmark_apply_undo"]
pub extern "C" fn rs_extmark_apply_undo(undo_info: ExtmarkUndoObject, undo: bool) {
    extmark_apply_undo(undo_info, undo);
}

/// Apply undo or redo for an extmark operation (internal implementation).
pub fn extmark_apply_undo(undo_info: ExtmarkUndoObject, undo: bool) {
    let curbuf = get_curbuf();

    match undo_info.type_ {
        UndoObjectType::Splice => {
            let splice = unsafe { undo_info.data.splice };
            if undo {
                extmark_splice_impl(
                    curbuf,
                    splice.start_row,
                    splice.start_col,
                    splice.start_byte,
                    splice.new_row,
                    splice.new_col,
                    splice.new_byte,
                    splice.old_row,
                    splice.old_col,
                    splice.old_byte,
                    ExtmarkOp::NoUndo,
                );
            } else {
                extmark_splice_impl(
                    curbuf,
                    splice.start_row,
                    splice.start_col,
                    splice.start_byte,
                    splice.old_row,
                    splice.old_col,
                    splice.old_byte,
                    splice.new_row,
                    splice.new_col,
                    splice.new_byte,
                    ExtmarkOp::NoUndo,
                );
            }
        }
        UndoObjectType::SavePos => {
            let pos = unsafe { undo_info.data.savepos };
            if undo && pos.old_row >= 0 {
                extmark_setraw(curbuf, pos.mark, pos.old_row, pos.old_col, pos.invalidated);
            }
            // No Redo since kExtmarkSplice will move marks back
        }
        UndoObjectType::Move => {
            let move_ = unsafe { undo_info.data.move_ };
            if undo {
                extmark_move_region(
                    curbuf,
                    move_.new_row,
                    move_.new_col as ColnrT,
                    move_.new_byte,
                    move_.extent_row,
                    move_.extent_col as ColnrT,
                    move_.extent_byte,
                    move_.start_row,
                    move_.start_col as ColnrT,
                    move_.start_byte,
                    ExtmarkOp::NoUndo,
                );
            } else {
                extmark_move_region(
                    curbuf,
                    move_.start_row,
                    move_.start_col as ColnrT,
                    move_.start_byte,
                    move_.extent_row,
                    move_.extent_col as ColnrT,
                    move_.extent_byte,
                    move_.new_row,
                    move_.new_col as ColnrT,
                    move_.new_byte,
                    ExtmarkOp::NoUndo,
                );
            }
        }
        _ => {}
    }
}

/// Set raw position of a mark (used during undo).
fn extmark_setraw(buf: BufHandle, mark_id: u64, row: c_int, col: ColnrT, invalid: bool) {
    let itr = unsafe { nvim_marktree_itr_alloc() };
    let tree = unsafe { nvim_buf_get_marktree(buf) };
    let key = unsafe { nvim_marktree_lookup(tree, mark_id, itr) };

    let do_move = key.pos.row != row || key.pos.col != col;
    if key.pos.row < 0 || (!do_move && !invalid) {
        unsafe { nvim_marktree_itr_free(itr) };
        return; // Mark was deleted or no change needed
    }

    // Only the position before undo needs to be redrawn here
    if !invalid && mt_decor_any(key) && key.pos.row != row {
        let (decor_data, ext) = mt_decor(key);
        unsafe { nvim_decor_redraw(buf, key.pos.row, key.pos.row, key.pos.col, decor_data, ext) };
    }

    let mut row1 = 0;
    let mut row2 = 0;

    let alt_itr = unsafe { nvim_marktree_itr_alloc() };
    unsafe { nvim_marktree_itr_copy(alt_itr, itr) };
    let alt = unsafe { nvim_marktree_get_alt(tree, key, alt_itr) };

    if invalid {
        let flags = unsafe { nvim_mt_itr_rawkey_get_flags(itr) };
        unsafe { nvim_mt_itr_rawkey_set_flags(itr, flags & !MT_FLAG_INVALID) };
        let alt_flags = unsafe { nvim_mt_itr_rawkey_get_flags(alt_itr) };
        unsafe { nvim_mt_itr_rawkey_set_flags(alt_itr, alt_flags & !MT_FLAG_INVALID) };

        let (revise_itr, revise_key) = if mt_end(key) {
            (alt_itr, alt)
        } else {
            (itr, key)
        };
        unsafe { nvim_marktree_revise_meta(tree, revise_itr, revise_key) };
    } else if !mt_invalid(key)
        && (key.flags & MT_FLAG_DECOR_SIGNTEXT != 0)
        && unsafe { nvim_buf_signcols_get_autom(buf) }
    {
        row1 = alt.pos.row.min(key.pos.row.min(row));
        row2 = alt.pos.row.max(key.pos.row.max(row));
        let line_count = unsafe { nvim_buf_get_ml_line_count(buf) as c_int };
        unsafe { nvim_buf_signcols_count_range(buf, row1, (line_count - 1).min(row2), 0, 1) };
        // kTrue = 1
    }

    if do_move {
        unsafe { nvim_marktree_move(tree, itr, row, col) };
    }

    if invalid {
        let (decor_data, ext) = mt_decor(key);
        unsafe {
            nvim_buf_put_decor(
                buf,
                decor_data,
                ext,
                row.min(key.pos.row),
                row.max(key.pos.row),
            )
        };
    } else if !mt_invalid(key)
        && (key.flags & MT_FLAG_DECOR_SIGNTEXT != 0)
        && unsafe { nvim_buf_signcols_get_autom(buf) }
    {
        let line_count = unsafe { nvim_buf_get_ml_line_count(buf) as c_int };
        unsafe { nvim_buf_signcols_count_range(buf, row1, (line_count - 1).min(row2), 0, 0) };
        // kNone = 0
    }

    unsafe { nvim_marktree_itr_free(alt_itr) };
    unsafe { nvim_marktree_itr_free(itr) };
}

/// Move a region of marks.
#[export_name = "extmark_move_region"]
pub extern "C" fn rs_extmark_move_region(
    buf: BufHandle,
    start_row: c_int,
    start_col: ColnrT,
    start_byte: BcountT,
    extent_row: c_int,
    extent_col: ColnrT,
    extent_byte: BcountT,
    new_row: c_int,
    new_col: ColnrT,
    new_byte: BcountT,
    undo: c_int,
) {
    let undo_op = ExtmarkOp::from_c_int(undo).unwrap_or(ExtmarkOp::Noop);
    extmark_move_region(
        buf,
        start_row,
        start_col,
        start_byte,
        extent_row,
        extent_col,
        extent_byte,
        new_row,
        new_col,
        new_byte,
        undo_op,
    );
}

/// Move a region of marks (internal implementation).
pub fn extmark_move_region(
    buf: BufHandle,
    start_row: c_int,
    start_col: ColnrT,
    start_byte: BcountT,
    extent_row: c_int,
    extent_col: ColnrT,
    extent_byte: BcountT,
    new_row: c_int,
    new_col: ColnrT,
    new_byte: BcountT,
    undo: ExtmarkOp,
) {
    unsafe { nvim_buf_set_deleted_bytes2(buf, 0) };

    // Send splice for deletion
    unsafe {
        nvim_buf_updates_send_splice(
            buf,
            start_row,
            start_col,
            start_byte,
            extent_row,
            extent_col,
            extent_byte,
            0,
            0,
            0,
        )
    };

    let row1 = start_row.min(new_row);
    let row2 = start_row.max(new_row) + extent_row;
    unsafe { nvim_buf_signcols_count_range(buf, row1, row2, 0, 1) }; // kTrue = 1

    let tree = unsafe { nvim_buf_get_marktree(buf) };
    unsafe {
        nvim_marktree_move_region(
            tree, start_row, start_col, extent_row, extent_col, new_row, new_col,
        )
    };

    unsafe { nvim_buf_signcols_count_range(buf, row1, row2, 0, 0) }; // kNone = 0

    // Send splice for insertion
    unsafe {
        nvim_buf_updates_send_splice(
            buf,
            new_row,
            new_col,
            new_byte,
            0,
            0,
            0,
            extent_row,
            extent_col,
            extent_byte,
        )
    };

    if undo == ExtmarkOp::Undo {
        let uhp = unsafe { nvim_u_force_get_undo_header(buf) };
        if uhp.is_null() {
            return;
        }

        let uvp = unsafe { nvim_uhp_get_extmark(uhp) };

        let move_ = ExtmarkMove {
            start_row,
            start_col: start_col as c_int,
            start_byte,
            extent_row,
            extent_col: extent_col as c_int,
            extent_byte,
            new_row,
            new_col: new_col as c_int,
            new_byte,
        };

        let obj = ExtmarkUndoObject {
            type_: UndoObjectType::Move,
            data: ExtmarkUndoData { move_ },
        };

        unsafe { nvim_extmark_undo_vec_push(uvp, obj) };
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mt_flags() {
        let flags = mt_flags(true, false, false, false);
        assert!(flags & MT_FLAG_RIGHT_GRAVITY != 0);
        assert!(flags & MT_FLAG_NO_UNDO == 0);

        let flags2 = mt_flags(false, true, true, true);
        assert!(flags2 & MT_FLAG_RIGHT_GRAVITY == 0);
        assert!(flags2 & MT_FLAG_NO_UNDO != 0);
        assert!(flags2 & MT_FLAG_INVALIDATE != 0);
        assert!(flags2 & MT_FLAG_DECOR_EXT != 0);
    }

    #[test]
    fn test_mt_key_helpers() {
        let key = MTKey {
            pos: MTPos::new(10, 5),
            ns: 1,
            id: 42,
            flags: MT_FLAG_PAIRED | MT_FLAG_END | MT_FLAG_RIGHT_GRAVITY,
            decor_data: DecorInlineData::zero(),
        };

        assert!(mt_paired(key));
        assert!(mt_end(key));
        assert!(mt_right(key));
        assert!(!mt_invalid(key));
        assert!(!mt_invalidate(key));
        assert!(!mt_no_undo(key));
        assert!(!mt_decor_any(key));
    }

    #[test]
    fn test_extmark_op_from_c_int() {
        assert_eq!(ExtmarkOp::from_c_int(0), Some(ExtmarkOp::Noop));
        assert_eq!(ExtmarkOp::from_c_int(1), Some(ExtmarkOp::Undo));
        assert_eq!(ExtmarkOp::from_c_int(2), Some(ExtmarkOp::NoUndo));
        assert_eq!(ExtmarkOp::from_c_int(3), Some(ExtmarkOp::UndoNoRedo));
        assert_eq!(ExtmarkOp::from_c_int(99), None);
    }

    #[test]
    fn test_mt_pair() {
        let start = MTKey {
            pos: MTPos::new(1, 0),
            ns: 1,
            id: 1,
            flags: 0,
            decor_data: DecorInlineData::zero(),
        };
        let end = MTKey {
            pos: MTPos::new(5, 10),
            ns: 1,
            id: 1,
            flags: MT_FLAG_END,
            decor_data: DecorInlineData::zero(),
        };
        let pair = MTPair::from_keys(start, end);
        assert_eq!(pair.start.pos.row, 1);
        assert_eq!(pair.end_pos.row, 5);
    }

    #[test]
    fn test_extmark_info_default() {
        let info = ExtmarkInfo::default();
        assert_eq!(info.row, 0);
        assert_eq!(info.end_row, 0);
        assert_eq!(info.ns_id, 0);
        assert_eq!(info.mark_id, 0);
    }
}

// ============================================================================
// Query Functions for API Layer
// ============================================================================

/// Extended information about an extmark for queries.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ExtmarkInfo {
    /// Namespace ID
    pub ns_id: u32,
    /// Mark ID
    pub mark_id: u32,
    /// Row position (-1 if not found)
    pub row: c_int,
    /// Column position
    pub col: ColnrT,
    /// End row (-1 if not paired)
    pub end_row: c_int,
    /// End column
    pub end_col: ColnrT,
    /// Mark flags
    pub flags: u16,
    /// Decoration data
    pub decor_data: DecorInlineData,
}

impl ExtmarkInfo {
    /// Create from a mark key.
    #[must_use]
    pub fn from_key(key: MTKey) -> Self {
        Self {
            ns_id: key.ns,
            mark_id: key.id,
            row: key.pos.row,
            col: key.pos.col,
            end_row: -1,
            end_col: 0,
            flags: key.flags,
            decor_data: key.decor_data,
        }
    }

    /// Create from a start key and end position.
    #[must_use]
    pub fn from_pair_pos(start: MTKey, end_pos: MTPos) -> Self {
        Self {
            ns_id: start.ns,
            mark_id: start.id,
            row: start.pos.row,
            col: start.pos.col,
            end_row: if end_pos.row >= 0 { end_pos.row } else { -1 },
            end_col: end_pos.col,
            flags: start.flags,
            decor_data: start.decor_data,
        }
    }
}

/// FFI export: Get extended extmark info by ID.
///
/// Returns an ExtmarkInfo structure with row = -1 if not found.
#[no_mangle]
pub extern "C" fn rs_extmark_get_by_id(buf: BufHandle, ns_id: u32, id: u32) -> ExtmarkInfo {
    let pair = extmark_from_id(buf, ns_id, id);

    if pair.start.id == 0 {
        ExtmarkInfo::default()
    } else if pair.end_pos.row >= 0 {
        ExtmarkInfo::from_pair_pos(pair.start, pair.end_pos)
    } else {
        ExtmarkInfo::from_key(pair.start)
    }
}

/// FFI export: Check if an extmark exists.
#[no_mangle]
pub extern "C" fn rs_extmark_exists(buf: BufHandle, ns_id: u32, id: u32) -> bool {
    let pair = extmark_from_id(buf, ns_id, id);
    pair.start.id != 0
}

/// FFI export: Get the next available ID for a namespace.
///
/// This is useful for auto-generating IDs when creating new extmarks.
#[no_mangle]
pub extern "C" fn rs_extmark_next_id(buf: BufHandle, ns_id: u32) -> u32 {
    let ns = unsafe { nvim_extmark_ns_get_ref(buf, ns_id) };
    if ns.is_null() {
        1
    } else {
        unsafe { *ns + 1 }
    }
}

// ============================================================================
// API Layer Functions
// ============================================================================

/// Query parameters for getting extmarks.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ExtmarkQueryParams {
    /// Namespace ID (0 for all)
    pub ns_id: u32,
    /// Start row
    pub start_row: c_int,
    /// Start column
    pub start_col: ColnrT,
    /// End row (-1 for no limit)
    pub end_row: c_int,
    /// End column (-1 for no limit)
    pub end_col: ColnrT,
    /// Maximum number of results (0 for unlimited)
    pub limit: u32,
    /// Type filter (bitmask of ExtmarkType)
    pub type_filter: u32,
    /// Whether to include details in results
    pub details: bool,
    /// Whether to overlap the search range
    pub overlap: bool,
}

impl Default for ExtmarkQueryParams {
    fn default() -> Self {
        Self {
            ns_id: 0,
            start_row: 0,
            start_col: 0,
            end_row: -1,
            end_col: -1,
            limit: 0,
            type_filter: 0,
            details: false,
            overlap: false,
        }
    }
}

/// FFI: Create default query params.
#[no_mangle]
pub extern "C" fn rs_extmark_query_params_default() -> ExtmarkQueryParams {
    ExtmarkQueryParams::default()
}

/// FFI: Check if query params are valid.
#[no_mangle]
pub extern "C" fn rs_extmark_query_params_valid(params: &ExtmarkQueryParams) -> c_int {
    // Start must be non-negative
    if params.start_row < 0 || params.start_col < 0 {
        return 0;
    }
    // End can be -1 (unlimited) or >= start
    if params.end_row >= 0 && params.end_row < params.start_row {
        return 0;
    }
    if params.end_row == params.start_row
        && params.end_col >= 0
        && params.end_col < params.start_col
    {
        return 0;
    }
    1
}

/// Result of counting extmarks.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ExtmarkCountResult {
    /// Number of extmarks found
    pub count: u32,
    /// Whether the limit was reached
    pub limit_reached: bool,
}

/// FFI: Count extmarks in a range.
#[no_mangle]
pub extern "C" fn rs_extmark_count_range(
    buf: BufHandle,
    ns_id: u32,
    start_row: c_int,
    start_col: ColnrT,
    end_row: c_int,
    end_col: ColnrT,
    limit: u32,
) -> ExtmarkCountResult {
    if unsafe { nvim_extmark_ns_map_size(buf) } == 0 {
        return ExtmarkCountResult::default();
    }

    let all_ns = ns_id == 0;
    let itr = unsafe { nvim_marktree_itr_alloc() };
    let tree = unsafe { nvim_buf_get_marktree(buf) };
    unsafe { nvim_marktree_itr_get(tree, start_row, start_col, itr) };

    let mut count: u32 = 0;
    let mut limit_reached = false;

    loop {
        let mark = unsafe { nvim_marktree_itr_current(itr) };

        // Check bounds
        if mark.pos.row < 0 {
            break;
        }
        if end_row >= 0 && mark.pos.row > end_row {
            break;
        }
        if end_row >= 0 && mark.pos.row == end_row && end_col >= 0 && mark.pos.col > end_col {
            break;
        }

        // Check namespace
        if all_ns || mark.ns == ns_id {
            // Don't count end marks of pairs
            if !mt_end(mark) {
                count += 1;
                if limit > 0 && count >= limit {
                    limit_reached = true;
                    break;
                }
            }
        }

        unsafe { nvim_marktree_itr_next(tree, itr) };
    }

    unsafe { nvim_marktree_itr_free(itr) };

    ExtmarkCountResult {
        count,
        limit_reached,
    }
}

/// FFI: Get namespace ID count for a buffer.
#[no_mangle]
pub extern "C" fn rs_extmark_ns_count(buf: BufHandle) -> usize {
    unsafe { nvim_extmark_ns_map_size(buf) }
}

/// FFI: Check if namespace has any extmarks in buffer.
#[no_mangle]
pub extern "C" fn rs_extmark_ns_has_marks(buf: BufHandle, ns_id: u32) -> bool {
    let ns = unsafe { nvim_extmark_ns_get_ref(buf, ns_id) };
    !ns.is_null()
}

/// Position for iteration bounds.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct ExtmarkBounds {
    /// Minimum row
    pub min_row: c_int,
    /// Maximum row
    pub max_row: c_int,
    /// Number of extmarks
    pub count: u32,
}

/// FFI: Get bounds of extmarks in a namespace.
#[no_mangle]
pub extern "C" fn rs_extmark_get_bounds(buf: BufHandle, ns_id: u32) -> ExtmarkBounds {
    if unsafe { nvim_extmark_ns_map_size(buf) } == 0 {
        return ExtmarkBounds {
            min_row: -1,
            max_row: -1,
            count: 0,
        };
    }

    let itr = unsafe { nvim_marktree_itr_alloc() };
    let tree = unsafe { nvim_buf_get_marktree(buf) };
    unsafe { nvim_marktree_itr_get(tree, 0, 0, itr) };

    let mut min_row: c_int = -1;
    let mut max_row: c_int = -1;
    let mut count: u32 = 0;

    loop {
        let mark = unsafe { nvim_marktree_itr_current(itr) };
        if mark.pos.row < 0 {
            break;
        }

        if (ns_id == 0 || mark.ns == ns_id) && !mt_end(mark) {
            if min_row < 0 || mark.pos.row < min_row {
                min_row = mark.pos.row;
            }
            if mark.pos.row > max_row {
                max_row = mark.pos.row;
            }
            count += 1;
        }

        unsafe { nvim_marktree_itr_next(tree, itr) };
    }

    unsafe { nvim_marktree_itr_free(itr) };

    ExtmarkBounds {
        min_row,
        max_row,
        count,
    }
}

/// FFI: Delete an extmark by ID (wrapper for API).
#[no_mangle]
pub extern "C" fn rs_nvim_buf_del_extmark(buf: BufHandle, ns_id: u32, id: u32) -> bool {
    rs_extmark_del_id(buf, ns_id, id)
}
