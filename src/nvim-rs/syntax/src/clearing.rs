//! Syntax clearing/teardown operations.
//!
//! Migrated from syntax_accessors.c: syn_clear_pattern, syn_clear_cluster,
//! syn_remove_pattern, syn_clear_time, syn_clear_one.

use std::ffi::{c_char, c_int, c_void};

use crate::types::{SynBlockHandle, SynClusterHandle, SynPatHandle};

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    // Pattern accessors
    fn nvim_synblock_get_pattern(block: SynBlockHandle, idx: c_int) -> SynPatHandle;
    fn nvim_synblock_get_pattern_count(block: SynBlockHandle) -> c_int;
    fn nvim_synpat_get_type(pat: SynPatHandle) -> c_int;
    fn nvim_synpat_get_flags(pat: SynPatHandle) -> c_int;
    fn nvim_synpat_get_syn_id(pat: SynPatHandle) -> i16;
    fn nvim_synpat_get_syncing(pat: SynPatHandle) -> c_int;
    fn nvim_synpat_get_pattern(pat: SynPatHandle) -> *const c_char;
    fn nvim_synpat_get_prog(pat: SynPatHandle) -> *mut c_void;
    fn nvim_synpat_get_cont_list(pat: SynPatHandle) -> *mut i16;
    fn nvim_synpat_get_next_list(pat: SynPatHandle) -> *mut i16;
    fn nvim_synpat_get_cont_in_list(pat: SynPatHandle) -> *mut i16;

    // Cluster accessors
    fn nvim_synblock_get_cluster(block: SynBlockHandle, idx: c_int) -> SynClusterHandle;
    fn nvim_syncluster_get_name(cluster: SynClusterHandle) -> *const c_char;
    fn nvim_syncluster_get_name_u(cluster: SynClusterHandle) -> *const c_char;
    fn nvim_syncluster_get_list(cluster: SynClusterHandle) -> *mut i16;

    // Memory management
    fn nvim_syn_xfree(ptr: *mut c_void);
    fn nvim_syn_vim_regfree(ptr: *mut c_void);

    // New accessors added for clearing.rs
    fn nvim_synblock_set_pattern_count(block: SynBlockHandle, len: c_int);
    fn nvim_synblock_memmove_patterns(
        block: SynBlockHandle,
        dst_idx: c_int,
        src_idx: c_int,
        count: c_int,
    );
    fn nvim_synblock_dec_folditems(block: SynBlockHandle);

    // curwin synblock for syn_clear_one
    fn nvim_get_curwin_synblock() -> SynBlockHandle;

    // Hashtab for syn_clear_one
    fn nvim_synblock_get_keywtab(block: SynBlockHandle) -> *mut c_void;
    fn nvim_synblock_get_keywtab_ic(block: SynBlockHandle) -> *mut c_void;

    /// C-side helper: remove all keyword entries with given id from hashtab.
    /// Keeps HI2KE/KE2HIKEY pointer arithmetic in C.
    fn nvim_syn_clear_keyword_in_ht(id: c_int, ht: *mut c_void);
}

// SPTYPE_START = 2 (must match C define)
const SPTYPE_START: c_int = 2;
// HL_FOLD = 0x2000 (must match C define)
const HL_FOLD: c_int = 0x2000;

// =============================================================================
// Phase 1 implementations
// =============================================================================

/// Clear and free one syntax pattern's allocated fields.
///
/// # Safety
/// block and index must be valid. When clearing all patterns, must be called
/// from last to first.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_clear_pattern(block: SynBlockHandle, i: c_int) {
    let pat = nvim_synblock_get_pattern(block, i);
    if pat.is_null() {
        return;
    }

    nvim_syn_xfree(nvim_synpat_get_pattern(pat) as *mut c_void);
    nvim_syn_vim_regfree(nvim_synpat_get_prog(pat));

    // Only free sp_cont_list, sp_next_list, and sp_syn.cont_in_list for
    // the first start pattern of a group (i == 0 or prev is not SPTYPE_START).
    let free_lists = if i == 0 {
        true
    } else {
        let prev = nvim_synblock_get_pattern(block, i - 1);
        prev.is_null() || nvim_synpat_get_type(prev) != SPTYPE_START
    };

    if free_lists {
        nvim_syn_xfree(nvim_synpat_get_cont_list(pat).cast());
        nvim_syn_xfree(nvim_synpat_get_next_list(pat).cast());
        nvim_syn_xfree(nvim_synpat_get_cont_in_list(pat).cast());
    }
}

/// Clear and free one syntax cluster's name, name_u, and list.
///
/// # Safety
/// block and index must be valid.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_clear_cluster(block: SynBlockHandle, i: c_int) {
    let cluster = nvim_synblock_get_cluster(block, i);
    if cluster.is_null() {
        return;
    }
    nvim_syn_xfree(nvim_syncluster_get_name(cluster) as *mut c_void);
    nvim_syn_xfree(nvim_syncluster_get_name_u(cluster) as *mut c_void);
    nvim_syn_xfree(nvim_syncluster_get_list(cluster).cast());
}

/// Remove one pattern from the buffer's pattern list (compact with memmove).
///
/// # Safety
/// block and idx must be valid; idx < pattern_count.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_remove_pattern(block: SynBlockHandle, idx: c_int) {
    // Decrement fold item count if the pattern has HL_FOLD
    let pat = nvim_synblock_get_pattern(block, idx);
    if !pat.is_null() && (nvim_synpat_get_flags(pat) & HL_FOLD) != 0 {
        nvim_synblock_dec_folditems(block);
    }

    // Clear the pattern's allocated fields
    rs_syn_clear_pattern(block, idx);

    // Compact: memmove(spp, spp+1, sizeof(synpat_T) * (ga_len - idx - 1))
    let count = nvim_synblock_get_pattern_count(block) - idx - 1;
    if count > 0 {
        nvim_synblock_memmove_patterns(block, idx, idx + 1, count);
    }
    nvim_synblock_set_pattern_count(block, nvim_synblock_get_pattern_count(block) - 1);
}

/// Clear all patterns and keywords matching a specific group ID.
///
/// # Safety
/// Must be called from main thread. id must be a valid syntax group ID.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_clear_one(id: c_int, syncing: c_int) {
    let block = nvim_get_curwin_synblock();
    if block.is_null() {
        return;
    }

    // Clear keywords only when not ":syn sync clear group-name"
    if syncing == 0 {
        let ht = nvim_synblock_get_keywtab(block);
        if !ht.is_null() {
            nvim_syn_clear_keyword_in_ht(id, ht);
        }
        let ht_ic = nvim_synblock_get_keywtab_ic(block);
        if !ht_ic.is_null() {
            nvim_syn_clear_keyword_in_ht(id, ht_ic);
        }
    }

    // Clear patterns for "id", iterating from last to first
    let mut idx = nvim_synblock_get_pattern_count(block) - 1;
    while idx >= 0 {
        let pat = nvim_synblock_get_pattern(block, idx);
        if !pat.is_null()
            && nvim_synpat_get_syn_id(pat) as c_int == id
            && nvim_synpat_get_syncing(pat) == syncing
        {
            rs_syn_remove_pattern(block, idx);
        }
        idx -= 1;
    }
}
