//! Sign removal operations
//!
//! This module handles removing placed signs from buffers.

use std::ffi::{c_char, c_int};

use nvim_decoration::types::{DecorInline, DecorSignHighlight, MTKey, MTPair};

use crate::{LinenrT, SignBufHandle, NS_ALL, NS_GLOBAL, NS_INVALID};

// =============================================================================
// C Accessor Extern Declarations
// =============================================================================

extern "C" {
    // Namespace operations
    fn nvim_namespace_lookup(name: *const c_char) -> c_int;

    // Buffer sign operations
    fn nvim_buf_meta_total_sign_hl(buf: SignBufHandle) -> u64;
    fn nvim_buf_meta_total_sign_text(buf: SignBufHandle) -> u64;

    // Extmark deletion
    fn extmark_del_id(buf: SignBufHandle, ns: u32, id: u32) -> bool;
    fn nvim_extmark_del(buf: SignBufHandle, itr: *mut std::ffi::c_void, mark: MTKey, end: bool);

    // Marktree iteration
    fn nvim_buf_get_marktree(buf: SignBufHandle) -> *mut std::ffi::c_void;
    fn nvim_marktree_itr_alloc() -> *mut std::ffi::c_void;
    #[link_name = "xfree"]
    fn nvim_marktree_itr_free(itr: *mut std::ffi::c_void);
    fn nvim_mtitr_has_x(itr: *const std::ffi::c_void) -> bool;
    fn rs_marktree_itr_get_overlap(
        b: *mut std::ffi::c_void,
        row: c_int,
        col: c_int,
        itr: *mut std::ffi::c_void,
    ) -> bool;
    fn rs_marktree_itr_step_overlap(
        b: *mut std::ffi::c_void,
        itr: *mut std::ffi::c_void,
        pair: *mut MTPair,
    ) -> bool;
    fn rs_marktree_itr_get(
        b: *mut std::ffi::c_void,
        row: c_int,
        col: c_int,
        itr: *mut std::ffi::c_void,
    );
    fn rs_marktree_itr_current(itr: *mut std::ffi::c_void) -> MTKey;
    fn rs_marktree_itr_next(b: *mut std::ffi::c_void, itr: *mut std::ffi::c_void) -> bool;

    // Sign decoration lookup (for sorting)
    fn decor_find_sign(decor: DecorInline) -> *mut DecorSignHighlight;

    // Namespace filtering
    fn rs_group_get_ns(
        group: *const c_char,
        ns_lookup: extern "C" fn(*const c_char) -> c_int,
    ) -> i64;
}

// =============================================================================
// Phase 2: MTKey helper functions (duplicated from query.rs for remove.rs scope)
// =============================================================================

const MT_FLAG_DECOR_EXT: u16 = 1 << 7;
const MT_FLAG_DECOR_SIGNTEXT: u16 = 1 << 9;
const MT_FLAG_DECOR_SIGNHL: u16 = 1 << 10;
const MT_FLAG_END: u16 = 1 << 1;

#[inline]
fn mtkey_to_decor_inline(key: &MTKey) -> DecorInline {
    DecorInline {
        ext: (key.flags & MT_FLAG_DECOR_EXT) != 0,
        _pad: [0; 7],
        data: key.decor_data,
    }
}

#[inline]
fn mtkey_is_decor_sign(key: &MTKey) -> bool {
    (key.flags & (MT_FLAG_DECOR_SIGNTEXT | MT_FLAG_DECOR_SIGNHL)) != 0
}

#[inline]
fn mtkey_is_end(key: &MTKey) -> bool {
    (key.flags & MT_FLAG_END) != 0
}

/// Compare two MTKey signs for priority-based sorting (ascending row, descending priority/id).
#[allow(clippy::cast_possible_wrap)]
unsafe fn cmp_signs_for_delete(a: &MTKey, b: &MTKey) -> std::cmp::Ordering {
    if a.pos.row != b.pos.row {
        return a.pos.row.cmp(&b.pos.row);
    }
    let sha = decor_find_sign(mtkey_to_decor_inline(a));
    let shb = decor_find_sign(mtkey_to_decor_inline(b));
    let (prio_a, add_a) = if sha.is_null() {
        (0u16, 0i32)
    } else {
        ((*sha).priority, (*sha).sign_add_id)
    };
    let (prio_b, add_b) = if shb.is_null() {
        (0u16, 0i32)
    } else {
        ((*shb).priority, (*shb).sign_add_id)
    };
    if prio_a != prio_b {
        return prio_b.cmp(&prio_a);
    }
    if a.id != b.id {
        return b.id.cmp(&a.id);
    }
    add_b.cmp(&add_a)
}

// =============================================================================
// Sign Removal Utilities
// =============================================================================

/// Check if a buffer has any signs.
///
/// Returns true if the buffer has any sign highlights or sign text.
///
/// # Safety
///
/// `buf` must be a valid buffer handle.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_buf_has_signs(buf: SignBufHandle) -> bool {
    if buf.is_null() {
        return false;
    }
    let hl_count = nvim_buf_meta_total_sign_hl(buf);
    let text_count = nvim_buf_meta_total_sign_text(buf);
    (hl_count + text_count) > 0
}

/// Determine the deletion mode based on parameters.
///
/// Returns:
/// - 0: Delete single sign by ID
/// - 1: Delete multiple signs (by line, group=*, or ID=0)
/// - -1: Invalid parameters
#[no_mangle]
pub extern "C" fn rs_sign_delete_mode(id: c_int, atlnum: LinenrT, group_is_star: bool) -> c_int {
    if id == 0 || atlnum > 0 || group_is_star {
        1 // Delete multiple signs
    } else if id > 0 {
        0 // Delete single sign by ID
    } else {
        -1 // Invalid
    }
}

/// Check if a group string represents "all groups" (is "*").
///
/// # Safety
///
/// `group` must be null or a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_group_is_star(group: *const c_char) -> bool {
    if group.is_null() {
        return false;
    }
    let first_byte = *group.cast::<u8>();
    first_byte == b'*'
}

/// Get namespace for sign removal operations.
///
/// Returns:
/// - NS_GLOBAL (0) for null group
/// - NS_ALL (UINT32_MAX) for "*" group
/// - namespace ID for named groups
/// - NS_INVALID (-1) for non-existing namespace
///
/// # Safety
///
/// `group` must be null or a valid null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_removal_namespace(group: *const c_char) -> i64 {
    if group.is_null() {
        return NS_GLOBAL;
    }

    let first_byte = *group.cast::<u8>();
    if first_byte == b'*' {
        return NS_ALL;
    }

    let ns = nvim_namespace_lookup(group);
    if ns != 0 {
        i64::from(ns)
    } else {
        NS_INVALID
    }
}

/// Delete a single sign by namespace and ID.
///
/// Returns true if the sign was deleted, false otherwise.
///
/// # Safety
///
/// `buf` must be a valid buffer handle.
#[no_mangle]
pub unsafe extern "C" fn rs_sign_delete_by_id(buf: SignBufHandle, ns: u32, id: u32) -> bool {
    if buf.is_null() {
        return false;
    }
    extmark_del_id(buf, ns, id)
}

// =============================================================================
// Bulk Removal Parameters
// =============================================================================

/// Parameters for bulk sign removal
#[repr(C)]
pub struct SignRemoveParams {
    /// Namespace filter (0 = global, UINT32_MAX = all, or specific ns)
    pub ns: i64,
    /// Sign ID filter (0 = all signs)
    pub id: c_int,
    /// Line number filter (0 = all lines, > 0 = specific line)
    pub atlnum: LinenrT,
}

impl SignRemoveParams {
    /// Check if this matches all signs regardless of namespace
    #[inline]
    pub const fn matches_all_namespaces(&self) -> bool {
        self.ns == NS_ALL
    }

    /// Check if this matches all sign IDs
    #[inline]
    pub const fn matches_all_ids(&self) -> bool {
        self.id == 0
    }

    /// Check if this matches all lines
    #[inline]
    pub const fn matches_all_lines(&self) -> bool {
        self.atlnum <= 0
    }

    /// Check if this matches a specific line
    #[inline]
    pub const fn has_line_filter(&self) -> bool {
        self.atlnum > 0
    }
}

/// Create removal parameters for global namespace.
#[no_mangle]
pub extern "C" fn rs_sign_remove_params_global(id: c_int, atlnum: LinenrT) -> SignRemoveParams {
    SignRemoveParams {
        ns: NS_GLOBAL,
        id,
        atlnum,
    }
}

/// Create removal parameters for all namespaces.
#[no_mangle]
pub extern "C" fn rs_sign_remove_params_all(id: c_int, atlnum: LinenrT) -> SignRemoveParams {
    SignRemoveParams {
        ns: NS_ALL,
        id,
        atlnum,
    }
}

/// Create removal parameters for a specific namespace.
#[no_mangle]
pub extern "C" fn rs_sign_remove_params_ns(
    ns: i64,
    id: c_int,
    atlnum: LinenrT,
) -> SignRemoveParams {
    SignRemoveParams { ns, id, atlnum }
}

// =============================================================================
// Core Sign Deletion
// =============================================================================

/// Callback used by rs_group_get_ns for namespace lookup.
extern "C" fn namespace_lookup_fn(name: *const c_char) -> c_int {
    unsafe { nvim_namespace_lookup(name) }
}

/// Delete signs from a buffer's marktree.
///
/// Replaces C `nvim_sign_delete_signs_impl`.
///
/// - If `atlnum > 0`: delete the highest-priority sign at that line number.
/// - If `atlnum == 0`: delete all signs matching ns/id.
///
/// Returns OK (1) on success, FAIL (0) if no sign was found at the given line.
///
/// # Safety
/// `buf` must be a valid buffer handle.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap
)]
#[unsafe(export_name = "nvim_sign_delete_signs_impl")]
pub unsafe extern "C" fn rs_nvim_sign_delete_signs_impl(
    buf: SignBufHandle,
    ns: i64,
    id: c_int,
    atlnum: LinenrT,
) -> c_int {
    const OK: c_int = 1;
    const FAIL: c_int = 0;

    let b = nvim_buf_get_marktree(buf);
    let row = if atlnum > 0 { atlnum - 1 } else { 0 };
    let mut signs: Vec<MTKey> = Vec::new();

    let itr = nvim_marktree_itr_alloc();

    if atlnum > 0 {
        // Collect all sign marks that span the given row via overlap iteration
        if !rs_marktree_itr_get_overlap(b, row, 0, itr) {
            nvim_marktree_itr_free(itr);
            return FAIL;
        }
        let mut pair = MTPair::default();
        while rs_marktree_itr_step_overlap(b, itr, &raw mut pair) {
            if (ns == NS_ALL || ns == i64::from(pair.start.ns)) && mtkey_is_decor_sign(&pair.start)
            {
                signs.push(pair.start);
            }
        }
        // Also iterate forward from overlap to collect non-overlapping signs at row
        while nvim_mtitr_has_x(itr) {
            let mark = rs_marktree_itr_current(itr);
            if mark.pos.row > row {
                break;
            }
            if !mtkey_is_end(&mark)
                && mtkey_is_decor_sign(&mark)
                && (id == 0 || mark.id as c_int == id)
                && (ns == NS_ALL || ns == i64::from(mark.ns))
            {
                signs.push(mark);
            }
            rs_marktree_itr_next(b, itr);
        }
        nvim_marktree_itr_free(itr);
        if signs.is_empty() {
            return FAIL;
        }
        // Sort and delete highest-priority sign
        signs.sort_by(|a, bk| cmp_signs_for_delete(a, bk));
        extmark_del_id(buf, signs[0].ns, signs[0].id);
    } else {
        // Delete all matching signs
        rs_marktree_itr_get(b, 0, 0, itr);
        while nvim_mtitr_has_x(itr) {
            let mark = rs_marktree_itr_current(itr);
            if !mtkey_is_end(&mark)
                && mtkey_is_decor_sign(&mark)
                && (id == 0 || mark.id as c_int == id)
                && (ns == NS_ALL || ns == i64::from(mark.ns))
            {
                nvim_extmark_del(buf, itr, mark, true);
            } else {
                rs_marktree_itr_next(b, itr);
            }
        }
        nvim_marktree_itr_free(itr);
    }

    OK
}

/// Delete the specified sign(s) from a buffer.
///
/// Resolves the namespace from group name and calls the Rust implementation.
///
/// Returns OK (1) on success, FAIL (0) on failure.
///
/// # Safety
/// `buf` must be a valid buffer handle. `group` must be null or valid C string.
#[no_mangle]
pub unsafe extern "C" fn rs_buf_delete_signs(
    buf: SignBufHandle,
    group: *const c_char,
    id: c_int,
    atlnum: LinenrT,
) -> c_int {
    let ns = rs_group_get_ns(group, namespace_lookup_fn);
    if ns < 0 {
        return 0; // FAIL
    }

    rs_nvim_sign_delete_signs_impl(buf, ns, id, atlnum)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_delete_mode() {
        // Single sign deletion
        assert_eq!(rs_sign_delete_mode(1, 0, false), 0);
        assert_eq!(rs_sign_delete_mode(100, 0, false), 0);

        // Multiple sign deletion
        assert_eq!(rs_sign_delete_mode(0, 0, false), 1); // ID = 0
        assert_eq!(rs_sign_delete_mode(1, 10, false), 1); // atlnum > 0
        assert_eq!(rs_sign_delete_mode(1, 0, true), 1); // group = *

        // Invalid
        assert_eq!(rs_sign_delete_mode(-1, 0, false), -1);
    }

    #[test]
    fn test_sign_remove_params() {
        let params = rs_sign_remove_params_global(5, 10);
        assert_eq!(params.ns, NS_GLOBAL);
        assert_eq!(params.id, 5);
        assert_eq!(params.atlnum, 10);
        assert!(!params.matches_all_namespaces());
        assert!(!params.matches_all_ids());
        assert!(params.has_line_filter());

        let params = rs_sign_remove_params_all(0, 0);
        assert!(params.matches_all_namespaces());
        assert!(params.matches_all_ids());
        assert!(params.matches_all_lines());
    }
}
