//! Syntax cluster management.
//!
//! This module handles:
//! - Cluster definition and storage
//! - Cluster membership operations
//! - Cluster-based contains/containedin logic

use std::ffi::{c_char, c_int};

use crate::types::{
    IdListHandle, SynBlockHandle, SynClusterHandle, CLUSTER_ADD, CLUSTER_REPLACE, CLUSTER_SUBTRACT,
    SYNID_ALLBUT, SYNID_CLUSTER,
};

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
