//! Syntax cluster management.
//!
//! This module handles:
//! - Cluster definition and storage
//! - Cluster membership operations
//! - Cluster-based contains/containedin logic

use std::ffi::{c_char, c_int, c_void};

use crate::types::{
    IdListHandle, SynBlockHandle, SynClusterHandle, CLUSTER_ADD, CLUSTER_REPLACE, CLUSTER_SUBTRACT,
    SYNID_ALLBUT, SYNID_CLUSTER,
};

extern "C" {
    fn xmalloc(size: usize) -> *mut c_void;
    fn xfree(ptr: *mut c_void);
}

// =============================================================================
// FFI declarations for cluster operations
// =============================================================================

extern "C" {
    // Cluster accessors
    fn nvim_syncluster_get_name(cluster: SynClusterHandle) -> *const c_char;
    fn nvim_syncluster_get_name_u(cluster: SynClusterHandle) -> *const c_char;
    fn nvim_syncluster_get_list(cluster: SynClusterHandle) -> IdListHandle;
    fn nvim_syncluster_has_list(cluster: SynClusterHandle) -> c_int;
    fn nvim_syncluster_get_id(cluster: SynClusterHandle) -> c_int;

    // Synblock cluster accessors
    fn nvim_synblock_get_cluster_count(block: SynBlockHandle) -> c_int;
    fn nvim_synblock_get_cluster(block: SynBlockHandle, idx: c_int) -> SynClusterHandle;
    fn nvim_synblock_get_cluster_id(block: SynBlockHandle, idx: c_int) -> c_int;
    fn nvim_synblock_get_spell_cluster_id(block: SynBlockHandle) -> c_int;
    fn nvim_synblock_get_nospell_cluster_id(block: SynBlockHandle) -> c_int;
    fn nvim_synblock_get_spell_cluster(block: SynBlockHandle) -> c_int;
    fn nvim_synblock_get_nospell_cluster(block: SynBlockHandle) -> c_int;

    // ID list operations
    fn nvim_id_list_first(list: IdListHandle) -> i16;
    fn nvim_id_list_get(list: IdListHandle, idx: c_int) -> i16;
    fn nvim_id_list_is_special(list: IdListHandle) -> c_int;
    fn nvim_id_list_count(list: IdListHandle) -> c_int;

    // Phase 32.3: Cluster lookup and containedin
    fn nvim_syn_cluster_name2id(name: *const c_char) -> c_int;
    fn nvim_synblock_has_containedin(block: SynBlockHandle) -> c_int;
    fn nvim_synblock_pattern_count(block: SynBlockHandle) -> c_int;
    fn nvim_synpat_get_inc_tag(pat: crate::types::SynPatHandle) -> c_int;
    fn nvim_synblock_is_spell_cluster(block: SynBlockHandle, id: c_int) -> c_int;
    fn nvim_synblock_is_nospell_cluster(block: SynBlockHandle, id: c_int) -> c_int;
}

// =============================================================================
// Cluster accessors
// =============================================================================

/// Get the name of a syntax cluster.
#[must_use]
pub fn cluster_name(cluster: SynClusterHandle) -> *const c_char {
    if cluster.is_null() {
        return std::ptr::null();
    }
    unsafe { nvim_syncluster_get_name(cluster) }
}

/// Get the uppercase name of a syntax cluster.
#[must_use]
pub fn cluster_name_upper(cluster: SynClusterHandle) -> *const c_char {
    if cluster.is_null() {
        return std::ptr::null();
    }
    unsafe { nvim_syncluster_get_name_u(cluster) }
}

/// Get the ID list for a syntax cluster.
#[must_use]
pub fn cluster_list(cluster: SynClusterHandle) -> IdListHandle {
    if cluster.is_null() {
        return IdListHandle::null();
    }
    unsafe { nvim_syncluster_get_list(cluster) }
}

/// Check if a cluster has a list of IDs.
#[must_use]
pub fn cluster_has_list(cluster: SynClusterHandle) -> bool {
    if cluster.is_null() {
        return false;
    }
    unsafe { nvim_syncluster_has_list(cluster) != 0 }
}

/// Get the ID of a cluster (SYNID_CLUSTER + index).
#[must_use]
pub fn cluster_id(cluster: SynClusterHandle) -> i32 {
    if cluster.is_null() {
        return 0;
    }
    unsafe { nvim_syncluster_get_id(cluster) }
}

// =============================================================================
// Synblock cluster accessors
// =============================================================================

/// Get the number of clusters in a synblock.
#[must_use]
pub fn synblock_cluster_count(block: SynBlockHandle) -> i32 {
    if block.is_null() {
        return 0;
    }
    unsafe { nvim_synblock_get_cluster_count(block) }
}

/// Get a cluster from a synblock by index.
#[must_use]
pub fn synblock_get_cluster(block: SynBlockHandle, idx: i32) -> SynClusterHandle {
    if block.is_null() {
        return SynClusterHandle::null();
    }
    unsafe { nvim_synblock_get_cluster(block, idx) }
}

/// Get the ID of a cluster by index (SYNID_CLUSTER + idx).
#[must_use]
pub fn synblock_cluster_id(block: SynBlockHandle, idx: i32) -> i32 {
    if block.is_null() {
        return 0;
    }
    unsafe { nvim_synblock_get_cluster_id(block, idx) }
}

/// Get the spell cluster ID for a synblock.
#[must_use]
pub fn synblock_spell_cluster_id(block: SynBlockHandle) -> i32 {
    if block.is_null() {
        return 0;
    }
    unsafe { nvim_synblock_get_spell_cluster_id(block) }
}

/// Get the nospell cluster ID for a synblock.
#[must_use]
pub fn synblock_nospell_cluster_id(block: SynBlockHandle) -> i32 {
    if block.is_null() {
        return 0;
    }
    unsafe { nvim_synblock_get_nospell_cluster_id(block) }
}

/// Get the spell cluster index for a synblock.
#[must_use]
pub fn synblock_spell_cluster(block: SynBlockHandle) -> i32 {
    if block.is_null() {
        return 0;
    }
    unsafe { nvim_synblock_get_spell_cluster(block) }
}

/// Get the nospell cluster index for a synblock.
#[must_use]
pub fn synblock_nospell_cluster(block: SynBlockHandle) -> i32 {
    if block.is_null() {
        return 0;
    }
    unsafe { nvim_synblock_get_nospell_cluster(block) }
}

// =============================================================================
// ID list operations
// =============================================================================

/// Get the first item in an ID list.
#[must_use]
pub fn id_list_first(list: IdListHandle) -> i16 {
    if list.is_null() {
        return 0;
    }
    unsafe { nvim_id_list_first(list) }
}

/// Get an item from an ID list by index.
/// Note: No bounds checking is performed.
///
/// # Safety
/// The caller must ensure the index is within bounds.
#[must_use]
pub unsafe fn id_list_get(list: IdListHandle, idx: i32) -> i16 {
    if list.is_null() {
        return 0;
    }
    nvim_id_list_get(list, idx)
}

/// Check if an ID list starts with a special marker (ALLBUT/TOP/CONTAINED).
#[must_use]
pub fn id_list_is_special(list: IdListHandle) -> bool {
    if list.is_null() {
        return false;
    }
    unsafe { nvim_id_list_is_special(list) != 0 }
}

/// Count the number of items in an ID list (terminated by 0).
#[must_use]
pub fn id_list_count(list: IdListHandle) -> i32 {
    if list.is_null() {
        return 0;
    }
    unsafe { nvim_id_list_count(list) }
}

// =============================================================================
// Cluster ID helpers
// =============================================================================

/// Check if an ID is a cluster ID.
#[must_use]
pub const fn is_cluster_id(id: i16) -> bool {
    id >= SYNID_CLUSTER as i16
}

/// Get the cluster index from a cluster ID.
#[must_use]
pub const fn cluster_index(id: i16) -> i32 {
    if is_cluster_id(id) {
        (id as i32) - SYNID_CLUSTER
    } else {
        -1
    }
}

/// Create a cluster ID from an index.
#[must_use]
pub const fn make_cluster_id(idx: i32) -> i16 {
    (SYNID_CLUSTER + idx) as i16
}

// =============================================================================
// ID list iteration
// =============================================================================

/// Iterator over items in an ID list.
pub struct IdListIter {
    list: IdListHandle,
    index: i32,
}

impl IdListIter {
    /// Create a new iterator over an ID list.
    #[must_use]
    pub fn new(list: IdListHandle) -> Self {
        Self { list, index: 0 }
    }
}

impl Iterator for IdListIter {
    type Item = i16;

    fn next(&mut self) -> Option<Self::Item> {
        if self.list.is_null() {
            return None;
        }
        let item = unsafe { nvim_id_list_get(self.list, self.index) };
        if item == 0 {
            None
        } else {
            self.index += 1;
            Some(item)
        }
    }
}

// =============================================================================
// Cluster operation types
// =============================================================================

/// Operations for modifying cluster membership.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClusterOp {
    /// Replace the cluster's contents.
    Replace,
    /// Add items to the cluster.
    Add,
    /// Remove items from the cluster.
    Subtract,
}

impl ClusterOp {
    /// Convert from a C integer constant.
    #[must_use]
    pub const fn from_c_int(val: i32) -> Option<Self> {
        match val {
            CLUSTER_REPLACE => Some(Self::Replace),
            CLUSTER_ADD => Some(Self::Add),
            CLUSTER_SUBTRACT => Some(Self::Subtract),
            _ => None,
        }
    }

    /// Convert to a C integer constant.
    #[must_use]
    pub const fn to_c_int(self) -> i32 {
        match self {
            Self::Replace => CLUSTER_REPLACE,
            Self::Add => CLUSTER_ADD,
            Self::Subtract => CLUSTER_SUBTRACT,
        }
    }
}

// =============================================================================
// Special ID markers
// =============================================================================

/// Check if an ID list represents "ALLBUT" (all groups except listed).
#[must_use]
pub fn id_list_is_allbut(list: IdListHandle) -> bool {
    if list.is_null() {
        return false;
    }
    let first = unsafe { nvim_id_list_first(list) };
    first == SYNID_ALLBUT as i16
}

/// The SYNID_ALLBUT marker value.
pub const ALLBUT_MARKER: i16 = SYNID_ALLBUT as i16;

/// The SYNID_CLUSTER base value.
pub const CLUSTER_BASE: i32 = SYNID_CLUSTER;

// =============================================================================
// Phase 32.3: Cluster lookup and containedin
// =============================================================================

/// Lookup a cluster by name and return its ID.
/// Returns 0 if not found.
///
/// # Safety
/// The name pointer must be a valid null-terminated C string.
#[must_use]
pub unsafe fn cluster_name_to_id(name: *const c_char) -> i32 {
    if name.is_null() {
        return 0;
    }
    nvim_syn_cluster_name2id(name)
}

/// Lookup a cluster by name (Rust string version).
/// Returns 0 if not found.
#[must_use]
pub fn cluster_lookup(name: &str) -> i32 {
    use std::ffi::CString;
    let Ok(cname) = CString::new(name) else {
        return 0;
    };
    unsafe { nvim_syn_cluster_name2id(cname.as_ptr()) }
}

/// Check if the synblock has any containedin items.
#[must_use]
pub fn synblock_has_containedin(block: SynBlockHandle) -> bool {
    if block.is_null() {
        return false;
    }
    unsafe { nvim_synblock_has_containedin(block) != 0 }
}

/// Get the pattern count for a synblock.
#[must_use]
pub fn synblock_pattern_count(block: SynBlockHandle) -> i32 {
    if block.is_null() {
        return 0;
    }
    unsafe { nvim_synblock_pattern_count(block) }
}

/// Get the inc_tag from a pattern.
#[must_use]
pub fn synpat_inc_tag(pat: crate::types::SynPatHandle) -> i32 {
    if pat.is_null() {
        return 0;
    }
    unsafe { nvim_synpat_get_inc_tag(pat) }
}

/// Check if a cluster ID is the @Spell cluster.
#[must_use]
pub fn is_spell_cluster(block: SynBlockHandle, id: i32) -> bool {
    if block.is_null() {
        return false;
    }
    unsafe { nvim_synblock_is_spell_cluster(block, id) != 0 }
}

/// Check if a cluster ID is the @NoSpell cluster.
#[must_use]
pub fn is_nospell_cluster(block: SynBlockHandle, id: i32) -> bool {
    if block.is_null() {
        return false;
    }
    unsafe { nvim_synblock_is_nospell_cluster(block, id) != 0 }
}

/// Check if an ID is a special spell-related cluster.
#[must_use]
pub fn is_spell_related_cluster(block: SynBlockHandle, id: i32) -> bool {
    is_spell_cluster(block, id) || is_nospell_cluster(block, id)
}

// =============================================================================
// FFI exports for cluster management (Phase Y3)
// =============================================================================

/// Opaque pointer to synblock for FFI
pub type SynBlockPtr = *const c_void;

/// Cluster membership result
#[repr(C)]
pub struct ClusterMembershipResult {
    /// Whether the ID is a member of the cluster
    pub is_member: c_int,
    /// Whether the cluster uses ALLBUT (inverted membership)
    pub is_allbut: c_int,
    /// The number of items in the cluster list
    pub list_count: c_int,
}
/// Cluster operation constants
#[no_mangle]
pub const extern "C" fn rs_cluster_op_replace() -> c_int {
    CLUSTER_REPLACE
}

#[no_mangle]
pub const extern "C" fn rs_cluster_op_add() -> c_int {
    CLUSTER_ADD
}

#[no_mangle]
pub const extern "C" fn rs_cluster_op_subtract() -> c_int {
    CLUSTER_SUBTRACT
}
/// Special ID constants
#[no_mangle]
pub const extern "C" fn rs_synid_allbut() -> i16 {
    SYNID_ALLBUT as i16
}

#[no_mangle]
pub const extern "C" fn rs_synid_cluster_base() -> c_int {
    SYNID_CLUSTER
}
/// Cluster info structure for queries
#[repr(C)]
pub struct ClusterInfo {
    /// The cluster ID (SYNID_CLUSTER + index)
    pub id: c_int,
    /// Number of items in the cluster list
    pub item_count: c_int,
    /// Whether the cluster uses ALLBUT
    pub is_allbut: c_int,
    /// Whether the cluster has any items
    pub is_empty: c_int,
}
/// Synblock cluster summary
#[repr(C)]
pub struct SynblockClusterSummary {
    /// Total number of clusters
    pub total_count: c_int,
    /// Number of empty clusters
    pub empty_count: c_int,
    /// Number of ALLBUT clusters
    pub allbut_count: c_int,
    /// Whether @Spell cluster exists
    pub has_spell: c_int,
    /// Whether @NoSpell cluster exists
    pub has_nospell: c_int,
}
// =============================================================================
// Phase 1: syn_combine_list migration
// =============================================================================

/// Merge/filter two 0-terminated int16_t ID lists according to a cluster
/// operation (Replace, Add, Subtract).
///
/// Both input lists are consumed (freed via xfree). Returns the new list
/// (allocated via xmalloc), or a null handle if the result is empty.
///
/// Matches the semantics of the C `syn_combine_list` function exactly.
///
/// # Safety
/// Both list handles must be null or point to xmalloc-allocated 0-terminated
/// int16_t arrays. After the call, both inputs are freed and must not be used.
unsafe fn combine_id_lists(
    list1: IdListHandle,
    list2: IdListHandle,
    op: ClusterOp,
) -> IdListHandle {
    // Handle degenerate cases.
    if list2.is_null() {
        // list2 is null: nothing to do, return list1 unchanged.
        return list1;
    }
    if list1.is_null() || op == ClusterOp::Replace {
        if op == ClusterOp::Replace {
            xfree(list1.0 as *mut c_void);
        }
        if op == ClusterOp::Replace || op == ClusterOp::Add {
            return list2;
        }
        // Subtract with null list1: free list2 and return null.
        xfree(list2.0 as *mut c_void);
        return IdListHandle::null();
    }

    // Count elements in both lists.
    let mut count1: usize = 0;
    let mut p = list1.0;
    while *p != 0 {
        count1 += 1;
        p = p.add(1);
    }

    let mut count2: usize = 0;
    let mut p = list2.0;
    while *p != 0 {
        count2 += 1;
        p = p.add(1);
    }

    // Sort both lists in place using Rust's sort (same semantics as qsort with
    // syn_compare_stub).
    let slice1 = std::slice::from_raw_parts_mut(list1.0, count1);
    let slice2 = std::slice::from_raw_parts_mut(list2.0, count2);
    slice1.sort_unstable();
    slice2.sort_unstable();

    // Two-pass merge: pass 1 counts elements, pass 2 populates the new list.
    let mut result_ptr: *mut i16 = std::ptr::null_mut();

    for round in 1..=2u32 {
        let mut g1 = list1.0;
        let mut g2 = list2.0;
        let mut count: usize = 0;

        // Merge while both lists have elements.
        while *g1 != 0 && *g2 != 0 {
            // Always take from list1 when it's smaller.
            if *g1 < *g2 {
                if round == 2 {
                    *result_ptr.add(count) = *g1;
                }
                count += 1;
                g1 = g1.add(1);
                continue;
            }
            // Take from list2 only for Add.
            if op == ClusterOp::Add {
                if round == 2 {
                    *result_ptr.add(count) = *g2;
                }
                count += 1;
            }
            if *g1 == *g2 {
                g1 = g1.add(1);
            }
            g2 = g2.add(1);
        }

        // Drain remaining from list1.
        while *g1 != 0 {
            if round == 2 {
                *result_ptr.add(count) = *g1;
            }
            count += 1;
            g1 = g1.add(1);
        }

        // Drain remaining from list2 (only for Add).
        if op == ClusterOp::Add {
            while *g2 != 0 {
                if round == 2 {
                    *result_ptr.add(count) = *g2;
                }
                count += 1;
                g2 = g2.add(1);
            }
        }

        if round == 1 {
            if count == 0 {
                // Empty result: no allocation needed.
                break;
            }
            // Allocate for count elements + terminating 0.
            result_ptr = xmalloc((count + 1) * std::mem::size_of::<i16>()) as *mut i16;
            *result_ptr.add(count) = 0;
        }
    }

    // Free both input lists and return the new one.
    xfree(list1.0 as *mut c_void);
    xfree(list2.0 as *mut c_void);

    if result_ptr.is_null() {
        IdListHandle::null()
    } else {
        IdListHandle(result_ptr)
    }
}

/// FFI export: combine two syntax cluster ID lists.
///
/// Reads `*clstr1` and `*clstr2`, calls `combine_id_lists`, writes the result
/// back to `*clstr1`, and sets `*clstr2` to null (matching C ownership).
///
/// # Safety
/// Both pointer arguments must be non-null pointers to IdListHandle values.
/// The lists they point to must be null or xmalloc-allocated 0-terminated
/// int16_t arrays.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_combine_list(
    clstr1: *mut IdListHandle,
    clstr2: *mut IdListHandle,
    list_op: c_int,
) {
    let l1 = *clstr1;
    let l2 = *clstr2;

    let op = match ClusterOp::from_c_int(list_op) {
        Some(op) => op,
        None => {
            // Unknown op: just free list2, leave list1 unchanged.
            if !l2.is_null() {
                xfree(l2.0 as *mut c_void);
            }
            *clstr2 = IdListHandle::null();
            return;
        }
    };

    let result = combine_id_lists(l1, l2, op);
    *clstr1 = result;
    *clstr2 = IdListHandle::null();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cluster_id_helpers() {
        // Test is_cluster_id
        assert!(!is_cluster_id(0));
        assert!(!is_cluster_id(100));
        assert!(is_cluster_id(SYNID_CLUSTER as i16));
        assert!(is_cluster_id((SYNID_CLUSTER + 1) as i16));
        assert!(is_cluster_id((SYNID_CLUSTER + 100) as i16));

        // Test cluster_index
        assert_eq!(cluster_index(0), -1);
        assert_eq!(cluster_index(100), -1);
        assert_eq!(cluster_index(SYNID_CLUSTER as i16), 0);
        assert_eq!(cluster_index((SYNID_CLUSTER + 1) as i16), 1);
        assert_eq!(cluster_index((SYNID_CLUSTER + 100) as i16), 100);

        // Test make_cluster_id
        assert_eq!(make_cluster_id(0), SYNID_CLUSTER as i16);
        assert_eq!(make_cluster_id(1), (SYNID_CLUSTER + 1) as i16);
        assert_eq!(make_cluster_id(100), (SYNID_CLUSTER + 100) as i16);
    }

    #[test]
    fn test_cluster_op() {
        assert_eq!(
            ClusterOp::from_c_int(CLUSTER_REPLACE),
            Some(ClusterOp::Replace)
        );
        assert_eq!(ClusterOp::from_c_int(CLUSTER_ADD), Some(ClusterOp::Add));
        assert_eq!(
            ClusterOp::from_c_int(CLUSTER_SUBTRACT),
            Some(ClusterOp::Subtract)
        );
        assert_eq!(ClusterOp::from_c_int(999), None);

        assert_eq!(ClusterOp::Replace.to_c_int(), CLUSTER_REPLACE);
        assert_eq!(ClusterOp::Add.to_c_int(), CLUSTER_ADD);
        assert_eq!(ClusterOp::Subtract.to_c_int(), CLUSTER_SUBTRACT);
    }

    #[test]
    fn test_null_handle_checks() {
        // Test null handle creation and checking
        // Note: Cannot call functions that use extern FFI even with null handles
        let null_cluster = SynClusterHandle::null();
        let null_block = SynBlockHandle(std::ptr::null_mut());
        let null_list = IdListHandle::null();

        assert!(null_cluster.is_null());
        assert!(null_block.is_null());
        assert!(null_list.is_null());

        // Non-null handle creation for testing
        let fake_ptr = std::ptr::dangling_mut::<std::ffi::c_void>();
        let non_null_cluster = SynClusterHandle(fake_ptr);
        assert!(!non_null_cluster.is_null());
    }
}
