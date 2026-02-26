//! Contained/Transparent logic for syntax highlighting.
//!
//! This module handles:
//! - contained, containedin, nextgroup handling
//! - Transparent group logic
//! - Syntax item containment checks

use std::ffi::c_int;

use crate::types::{
    IdListHandle, StateItemHandle, SynPatHandle, HL_CONTAINED, HL_INCLUDED_TOPLEVEL, HL_MATCH,
    HL_TRANSP, SYNID_ALLBUT, SYNID_CLUSTER, SYNID_CONTAINED, SYNID_TOP,
};

// =============================================================================
// FFI declarations for containment operations
// =============================================================================

extern "C" {
    // State item flag accessors
    fn nvim_stateitem_get_flags(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_set_flags(item: StateItemHandle, flags: c_int);
    fn nvim_stateitem_add_flags(item: StateItemHandle, flags: c_int);
    fn nvim_stateitem_or_flags(item: StateItemHandle, flags: c_int);
    fn nvim_stateitem_has_trans_cont(item: StateItemHandle) -> c_int;

    // State item list accessors
    fn nvim_stateitem_get_cont_list(item: StateItemHandle) -> IdListHandle;
    fn nvim_stateitem_has_cont_list(item: StateItemHandle) -> c_int;
    fn nvim_stateitem_set_cont_list(item: StateItemHandle, list: IdListHandle);

    // Pattern flag accessors
    fn nvim_synpat_get_flags(pat: SynPatHandle) -> c_int;

    // Pattern list accessors
    fn nvim_synpat_get_cont_list(pat: SynPatHandle) -> IdListHandle;
    fn nvim_synpat_get_next_list(pat: SynPatHandle) -> IdListHandle;
    fn nvim_synpat_get_cont_in_list(pat: SynPatHandle) -> IdListHandle;
    fn nvim_synpat_has_cont_list(pat: SynPatHandle) -> c_int;
    fn nvim_synpat_has_next_list(pat: SynPatHandle) -> c_int;
    fn nvim_synpat_has_cont_in_list(pat: SynPatHandle) -> c_int;

    // ID list check (old C delegation - kept for reference during migration)
    fn nvim_syn_in_id_list(
        cur_si: StateItemHandle,
        list: IdListHandle,
        id: c_int,
        inc_tag: c_int,
        cont_in_list: IdListHandle,
        flags: c_int,
    ) -> c_int;

    // New accessors for Phase 2: rs_syn_in_id_list implementation
    fn nvim_stateitem_get_idx(item: StateItemHandle) -> c_int;
    fn nvim_syn_get_pattern_sp_syn_id(idx: c_int) -> i16;
    fn nvim_syn_get_pattern_sp_syn_inc_tag(idx: c_int) -> c_int;
    fn nvim_syn_get_pattern_sp_syn_cont_in_list(idx: c_int) -> IdListHandle;
    fn nvim_syn_get_pattern_flags(idx: c_int) -> c_int;
    fn nvim_syn_get_cluster_scl_list(idx: c_int) -> IdListHandle;
    fn nvim_syn_is_id_list_all(list: IdListHandle) -> c_int;
}

// =============================================================================
// State item flag operations
// =============================================================================

/// Get the flags for a state item.
#[must_use]
pub fn stateitem_flags(item: StateItemHandle) -> i32 {
    if item.is_null() {
        return 0;
    }
    unsafe { nvim_stateitem_get_flags(item) }
}

/// Set the flags for a state item.
///
/// # Safety
/// The item must be a valid non-null pointer.
pub unsafe fn set_stateitem_flags(item: StateItemHandle, flags: i32) {
    if !item.is_null() {
        nvim_stateitem_set_flags(item, flags);
    }
}

/// Add flags to a state item (using |=).
///
/// # Safety
/// The item must be a valid non-null pointer.
pub unsafe fn add_stateitem_flags(item: StateItemHandle, flags: i32) {
    if !item.is_null() {
        nvim_stateitem_add_flags(item, flags);
    }
}

/// OR flags into a state item (same as add_flags).
///
/// # Safety
/// The item must be a valid non-null pointer.
pub unsafe fn or_stateitem_flags(item: StateItemHandle, flags: i32) {
    if !item.is_null() {
        nvim_stateitem_or_flags(item, flags);
    }
}

// =============================================================================
// Transparent item checks
// =============================================================================

/// Check if a state item has the HL_TRANS_CONT flag set.
/// This indicates a transparent item without a contains argument.
#[must_use]
pub fn stateitem_has_trans_cont(item: StateItemHandle) -> bool {
    if item.is_null() {
        return false;
    }
    unsafe { nvim_stateitem_has_trans_cont(item) != 0 }
}

/// Check if a state item is transparent (has HL_TRANSP flag).
#[must_use]
pub fn stateitem_is_transparent(item: StateItemHandle) -> bool {
    if item.is_null() {
        return false;
    }
    (stateitem_flags(item) & HL_TRANSP) != 0
}

/// Check if a state item has the HL_MATCH flag set.
#[must_use]
pub fn stateitem_is_match(item: StateItemHandle) -> bool {
    if item.is_null() {
        return false;
    }
    (stateitem_flags(item) & HL_MATCH) != 0
}

/// Check if a state item is contained (has HL_CONTAINED flag).
#[must_use]
pub fn stateitem_is_contained(item: StateItemHandle) -> bool {
    if item.is_null() {
        return false;
    }
    (stateitem_flags(item) & HL_CONTAINED) != 0
}

// =============================================================================
// State item list operations
// =============================================================================

/// Get the cont_list for a state item.
#[must_use]
pub fn stateitem_cont_list(item: StateItemHandle) -> IdListHandle {
    if item.is_null() {
        return IdListHandle::null();
    }
    unsafe { nvim_stateitem_get_cont_list(item) }
}

/// Check if a state item has a cont_list.
#[must_use]
pub fn stateitem_has_cont_list(item: StateItemHandle) -> bool {
    if item.is_null() {
        return false;
    }
    unsafe { nvim_stateitem_has_cont_list(item) != 0 }
}

/// Set the cont_list for a state item.
///
/// # Safety
/// The item must be a valid non-null pointer.
pub unsafe fn set_stateitem_cont_list(item: StateItemHandle, list: IdListHandle) {
    if !item.is_null() {
        nvim_stateitem_set_cont_list(item, list);
    }
}

// =============================================================================
// Pattern flag checks
// =============================================================================

/// Get the flags for a pattern.
#[must_use]
pub fn synpat_flags(pat: SynPatHandle) -> i32 {
    if pat.is_null() {
        return 0;
    }
    unsafe { nvim_synpat_get_flags(pat) }
}

/// Check if a pattern is transparent.
#[must_use]
pub fn synpat_is_transparent(pat: SynPatHandle) -> bool {
    if pat.is_null() {
        return false;
    }
    (synpat_flags(pat) & HL_TRANSP) != 0
}

/// Check if a pattern is contained.
#[must_use]
pub fn synpat_is_contained(pat: SynPatHandle) -> bool {
    if pat.is_null() {
        return false;
    }
    (synpat_flags(pat) & HL_CONTAINED) != 0
}

// =============================================================================
// Pattern list accessors
// =============================================================================

/// Get the cont_list for a pattern.
#[must_use]
pub fn synpat_cont_list(pat: SynPatHandle) -> IdListHandle {
    if pat.is_null() {
        return IdListHandle::null();
    }
    unsafe { nvim_synpat_get_cont_list(pat) }
}

/// Get the next_list for a pattern.
#[must_use]
pub fn synpat_next_list(pat: SynPatHandle) -> IdListHandle {
    if pat.is_null() {
        return IdListHandle::null();
    }
    unsafe { nvim_synpat_get_next_list(pat) }
}

/// Get the cont_in_list for a pattern.
#[must_use]
pub fn synpat_cont_in_list(pat: SynPatHandle) -> IdListHandle {
    if pat.is_null() {
        return IdListHandle::null();
    }
    unsafe { nvim_synpat_get_cont_in_list(pat) }
}

/// Check if a pattern has a cont_list.
#[must_use]
pub fn synpat_has_cont_list(pat: SynPatHandle) -> bool {
    if pat.is_null() {
        return false;
    }
    unsafe { nvim_synpat_has_cont_list(pat) != 0 }
}

/// Check if a pattern has a next_list.
#[must_use]
pub fn synpat_has_next_list(pat: SynPatHandle) -> bool {
    if pat.is_null() {
        return false;
    }
    unsafe { nvim_synpat_has_next_list(pat) != 0 }
}

/// Check if a pattern has a cont_in_list.
#[must_use]
pub fn synpat_has_cont_in_list(pat: SynPatHandle) -> bool {
    if pat.is_null() {
        return false;
    }
    unsafe { nvim_synpat_has_cont_in_list(pat) != 0 }
}

// =============================================================================
// Main containment check
// =============================================================================

/// Parameters for checking if an ID is in a list.
#[derive(Debug, Clone, Copy)]
pub struct IdListCheckParams {
    /// Syntax ID to check.
    pub id: i16,
    /// Include tag (for `:syn include` grouping).
    pub inc_tag: i32,
    /// Containedin list (may be null).
    pub cont_in_list: IdListHandle,
    /// Flags (HL_CONTAINED, etc.).
    pub flags: i32,
}

impl IdListCheckParams {
    /// Create new check parameters.
    #[must_use]
    pub fn new(id: i16, inc_tag: i32, cont_in_list: IdListHandle, flags: i32) -> Self {
        Self {
            id,
            inc_tag,
            cont_in_list,
            flags,
        }
    }

    /// Create check parameters with no containedin list.
    #[must_use]
    pub fn simple(id: i16, inc_tag: i32, flags: i32) -> Self {
        Self {
            id,
            inc_tag,
            cont_in_list: IdListHandle::null(),
            flags,
        }
    }
}

// =============================================================================
// Phase 2: Full in_id_list implementation in Rust
// =============================================================================

/// Core recursive implementation of the in_id_list check.
///
/// `depth` tracks the recursion level for cluster expansion (limit: 30).
///
/// # Safety
/// All handle arguments must be null or valid C pointers.
unsafe fn in_id_list_inner(
    cur_si: StateItemHandle,
    list: IdListHandle,
    id: i16,
    inc_tag: c_int,
    cont_in_list: IdListHandle,
    flags: c_int,
    depth: i32,
) -> bool {
    // If ssp has a "containedin" list and "cur_si" is in it, return true.
    if !cur_si.is_null() && !cont_in_list.is_null() && (flags & HL_MATCH) == 0 {
        // Walk back through transparent items without a contains argument.
        let actual_si = crate::state_ops::rs_stateitem_prev_if_trans_cont(cur_si);

        // cur_si->si_idx is -1 for keywords; keywords never contain anything.
        let si_idx = nvim_stateitem_get_idx(actual_si);
        if si_idx >= 0 {
            let pat_id = nvim_syn_get_pattern_sp_syn_id(si_idx);
            let pat_inc_tag = nvim_syn_get_pattern_sp_syn_inc_tag(si_idx);
            let pat_cont_in = nvim_syn_get_pattern_sp_syn_cont_in_list(si_idx);
            let pat_flags = nvim_syn_get_pattern_flags(si_idx);
            if in_id_list_inner(
                StateItemHandle(std::ptr::null_mut()),
                cont_in_list,
                pat_id,
                pat_inc_tag,
                pat_cont_in,
                pat_flags,
                depth,
            ) {
                return true;
            }
        }
    }

    if list.is_null() {
        return false;
    }

    // If list is ID_LIST_ALL, we are in a transparent item that isn't
    // inside anything. Only allow not-contained groups.
    if nvim_syn_is_id_list_all(list) != 0 {
        return (flags & HL_CONTAINED) == 0;
    }

    // Is this top-level (not 'contained') in the file it was declared in?
    let toplevel = (flags & HL_CONTAINED) == 0 || (flags & HL_INCLUDED_TOPLEVEL) != 0;

    // The first item may be a special marker (ALLBUT/TOP/CONTAINED).
    let mut list_ptr = list.0;
    let first_item = *list_ptr;
    let retval: bool;

    if first_item >= SYNID_ALLBUT as i16 && (first_item as c_int) < SYNID_CLUSTER {
        let item_int = first_item as c_int;
        if item_int < SYNID_TOP {
            // ALL or ALLBUT: accept all groups in the same file
            if item_int - SYNID_ALLBUT != inc_tag {
                return false;
            }
        } else if item_int < SYNID_CONTAINED {
            // TOP: accept all not-contained groups in the same file
            if item_int - SYNID_TOP != inc_tag || !toplevel {
                return false;
            }
        } else {
            // CONTAINED: accept all contained groups in the same file
            if item_int - SYNID_CONTAINED != inc_tag || toplevel {
                return false;
            }
        }
        list_ptr = list_ptr.add(1);
        retval = false;
    } else {
        retval = true;
    }

    // Return `retval` if id is in the contains list.
    let mut item = *list_ptr;
    while item != 0 {
        if item == id {
            return retval;
        }
        if (item as c_int) >= SYNID_CLUSTER {
            let cluster_idx = (item as c_int) - SYNID_CLUSTER;
            let scl_list = nvim_syn_get_cluster_scl_list(cluster_idx);
            // Restrict recursiveness to 30 to avoid an endless loop for a
            // cluster that includes itself (indirectly).
            if !scl_list.is_null() && depth < 30 {
                let r = in_id_list_inner(
                    StateItemHandle(std::ptr::null_mut()),
                    scl_list,
                    id,
                    inc_tag,
                    IdListHandle::null(),
                    flags,
                    depth + 1,
                );
                if r {
                    return retval;
                }
            }
        }
        list_ptr = list_ptr.add(1);
        item = *list_ptr;
    }
    !retval
}

/// Check if a syntax ID is in an ID list.
///
/// This is the core containment check used throughout syntax highlighting.
/// It handles:
/// - Direct ID membership
/// - ALLBUT lists (all except specified)
/// - TOP lists (top-level only)
/// - CONTAINED lists (contained only)
/// - Cluster expansion
/// - containedin checking
///
/// # Arguments
/// * `cur_si` - Current state item (may be null)
/// * `list` - ID list to check against
/// * `params` - Check parameters (id, inc_tag, cont_in_list, flags)
///
/// # Returns
/// true if the ID is considered to be "in" the list
#[must_use]
pub fn in_id_list(cur_si: StateItemHandle, list: IdListHandle, params: &IdListCheckParams) -> bool {
    unsafe {
        in_id_list_inner(
            cur_si,
            list,
            params.id,
            params.inc_tag,
            params.cont_in_list,
            params.flags,
            0,
        )
    }
}

/// FFI export: check if a syntax ID is in an ID list.
///
/// This is called from C wrappers (nvim_syn_in_id_list and in_id_list shim).
///
/// # Safety
/// All pointer arguments must be null or valid C pointers.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_in_id_list(
    cur_si: StateItemHandle,
    list: IdListHandle,
    id: c_int,
    inc_tag: c_int,
    cont_in_list: IdListHandle,
    flags: c_int,
) -> c_int {
    c_int::from(in_id_list_inner(
        cur_si,
        list,
        id as i16,
        inc_tag,
        cont_in_list,
        flags,
        0,
    ))
}

/// Check if a syntax ID is in an ID list (simple version).
///
/// Convenience wrapper when you don't have a cont_in_list.
#[must_use]
pub fn in_id_list_simple(
    cur_si: StateItemHandle,
    list: IdListHandle,
    id: i16,
    inc_tag: i32,
    flags: i32,
) -> bool {
    let params = IdListCheckParams::simple(id, inc_tag, flags);
    in_id_list(cur_si, list, &params)
}

// =============================================================================
// ID list type classification
// =============================================================================

/// Classification of special ID list markers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdListType {
    /// Normal ID list (no special marker).
    Normal,
    /// ALLBUT: accept all groups except those listed.
    AllBut { inc_tag: i16 },
    /// TOP: accept all top-level (not contained) groups.
    Top { inc_tag: i16 },
    /// CONTAINED: accept all contained groups.
    Contained { inc_tag: i16 },
}

impl IdListType {
    /// Classify an ID list based on its first element.
    ///
    /// # Arguments
    /// * `first_item` - The first item in the ID list
    #[must_use]
    pub fn classify(first_item: i16) -> Self {
        let item = first_item as i32;
        if item >= SYNID_ALLBUT && item < SYNID_CLUSTER {
            if item < SYNID_TOP {
                // ALLBUT
                Self::AllBut {
                    inc_tag: (item - SYNID_ALLBUT) as i16,
                }
            } else if item < SYNID_CONTAINED {
                // TOP
                Self::Top {
                    inc_tag: (item - SYNID_TOP) as i16,
                }
            } else {
                // CONTAINED
                Self::Contained {
                    inc_tag: (item - SYNID_CONTAINED) as i16,
                }
            }
        } else {
            Self::Normal
        }
    }

    /// Check if this is a special list type (not normal).
    #[must_use]
    pub fn is_special(&self) -> bool {
        !matches!(self, Self::Normal)
    }

    /// Get the include tag if this is a special list type.
    #[must_use]
    pub fn inc_tag(&self) -> Option<i16> {
        match self {
            Self::AllBut { inc_tag } | Self::Top { inc_tag } | Self::Contained { inc_tag } => {
                Some(*inc_tag)
            }
            Self::Normal => None,
        }
    }
}

// =============================================================================
// Toplevel check helpers
// =============================================================================

/// Check if a group is considered top-level.
///
/// A group is top-level if it's not contained, or if it was included
/// at the top level (has HL_INCLUDED_TOPLEVEL flag).
#[must_use]
pub const fn is_toplevel(flags: i32) -> bool {
    (flags & HL_CONTAINED) == 0 || (flags & HL_INCLUDED_TOPLEVEL) != 0
}

/// Check if a group can appear at the top level.
///
/// Groups without HL_CONTAINED can appear at the top level.
#[must_use]
pub const fn can_be_toplevel(flags: i32) -> bool {
    (flags & HL_CONTAINED) == 0
}

// =============================================================================
// Transparent handling helpers
// =============================================================================

/// Result of applying transparent item logic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransparentResult {
    /// Use the containing item's attributes.
    UseContaining,
    /// Use the item's own attributes.
    UseOwn,
}

/// Determine how to handle a transparent item's attributes.
///
/// If a state item is transparent and not a match item, it should
/// use the containing item's attributes instead of its own.
#[must_use]
pub fn transparent_attr_handling(item: StateItemHandle) -> TransparentResult {
    if item.is_null() {
        return TransparentResult::UseOwn;
    }
    let flags = stateitem_flags(item);
    if (flags & HL_TRANSP) != 0 && (flags & HL_MATCH) == 0 {
        TransparentResult::UseContaining
    } else {
        TransparentResult::UseOwn
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id_list_type_classify() {
        // Normal ID
        assert_eq!(IdListType::classify(100), IdListType::Normal);
        assert_eq!(IdListType::classify(0), IdListType::Normal);

        // ALLBUT range starts at SYNID_ALLBUT (MAX_HL_ID = 20000)
        let allbut_id = (SYNID_ALLBUT + 5) as i16;
        match IdListType::classify(allbut_id) {
            IdListType::AllBut { inc_tag } => assert_eq!(inc_tag, 5),
            _ => panic!("Expected AllBut"),
        }

        // TOP range starts at SYNID_TOP (21000)
        let top_id = (SYNID_TOP + 10) as i16;
        match IdListType::classify(top_id) {
            IdListType::Top { inc_tag } => assert_eq!(inc_tag, 10),
            _ => panic!("Expected Top"),
        }

        // CONTAINED range starts at SYNID_CONTAINED (22000)
        let contained_id = (SYNID_CONTAINED + 15) as i16;
        match IdListType::classify(contained_id) {
            IdListType::Contained { inc_tag } => assert_eq!(inc_tag, 15),
            _ => panic!("Expected Contained"),
        }
    }

    #[test]
    fn test_id_list_type_is_special() {
        assert!(!IdListType::Normal.is_special());
        assert!(IdListType::AllBut { inc_tag: 0 }.is_special());
        assert!(IdListType::Top { inc_tag: 0 }.is_special());
        assert!(IdListType::Contained { inc_tag: 0 }.is_special());
    }

    #[test]
    fn test_id_list_type_inc_tag() {
        assert_eq!(IdListType::Normal.inc_tag(), None);
        assert_eq!(IdListType::AllBut { inc_tag: 5 }.inc_tag(), Some(5));
        assert_eq!(IdListType::Top { inc_tag: 10 }.inc_tag(), Some(10));
        assert_eq!(IdListType::Contained { inc_tag: 15 }.inc_tag(), Some(15));
    }

    #[test]
    fn test_is_toplevel() {
        // Not contained = toplevel
        assert!(is_toplevel(0));

        // Contained = not toplevel
        assert!(!is_toplevel(HL_CONTAINED));

        // Contained but with INCLUDED_TOPLEVEL = toplevel
        assert!(is_toplevel(HL_CONTAINED | HL_INCLUDED_TOPLEVEL));

        // Other flags don't affect toplevel
        assert!(is_toplevel(HL_TRANSP));
        assert!(!is_toplevel(HL_CONTAINED | HL_TRANSP));
    }

    #[test]
    fn test_can_be_toplevel() {
        assert!(can_be_toplevel(0));
        assert!(!can_be_toplevel(HL_CONTAINED));
        // INCLUDED_TOPLEVEL doesn't affect can_be_toplevel
        assert!(!can_be_toplevel(HL_CONTAINED | HL_INCLUDED_TOPLEVEL));
        assert!(can_be_toplevel(HL_TRANSP));
    }

    #[test]
    fn test_null_handle_checks() {
        let null_item = StateItemHandle(std::ptr::null_mut());
        let null_pat = SynPatHandle(std::ptr::null_mut());

        assert!(null_item.is_null());
        assert!(null_pat.is_null());

        // Non-null handle
        let fake_ptr = std::ptr::dangling_mut::<std::ffi::c_void>();
        let non_null_item = StateItemHandle(fake_ptr);
        assert!(!non_null_item.is_null());
    }

    #[test]
    fn test_id_list_check_params() {
        let params = IdListCheckParams::new(100, 5, IdListHandle::null(), HL_CONTAINED);
        assert_eq!(params.id, 100);
        assert_eq!(params.inc_tag, 5);
        assert!(params.cont_in_list.is_null());
        assert_eq!(params.flags, HL_CONTAINED);

        let simple = IdListCheckParams::simple(200, 10, 0);
        assert_eq!(simple.id, 200);
        assert_eq!(simple.inc_tag, 10);
        assert!(simple.cont_in_list.is_null());
        assert_eq!(simple.flags, 0);
    }

    #[test]
    fn test_transparent_result() {
        // Test the enum variants
        assert_eq!(
            TransparentResult::UseContaining,
            TransparentResult::UseContaining
        );
        assert_eq!(TransparentResult::UseOwn, TransparentResult::UseOwn);
        assert_ne!(TransparentResult::UseContaining, TransparentResult::UseOwn);
    }
}
