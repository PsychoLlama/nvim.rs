//! Syntax clearing/teardown operations.
//!
//! Migrated from syntax_accessors.c: syn_clear_pattern, syn_clear_cluster,
//! syn_remove_pattern, syn_clear_one, syntax_clear, reset_synblock,
//! syntax_sync_clear.

use std::ffi::{c_char, c_int, c_void};

use crate::types::{KeyEntryHandle, SynBlockHandle, SynClusterHandle, SynPatHandle, WinHandle};

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    // Pattern accessors
    fn nvim_synblock_get_pattern(block: SynBlockHandle, idx: c_int) -> SynPatHandle;
    fn nvim_synblock_get_pattern_count(block: SynBlockHandle) -> c_int;

    // Cluster accessors
    fn nvim_synblock_get_cluster_count(block: SynBlockHandle) -> c_int;
    fn nvim_synblock_get_cluster(block: SynBlockHandle, idx: c_int) -> SynClusterHandle;

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
    fn nvim_syn_get_curwin_synblock() -> SynBlockHandle;

    // Hashtab for syn_clear_one
    fn nvim_synblock_get_keywtab(block: SynBlockHandle) -> *mut c_void;
    fn nvim_synblock_get_keywtab_ic(block: SynBlockHandle) -> *mut c_void;

    // Phase 15 bulk clearing accessors

    /// Full clear of a synblock (replaces 8 individual clearing functions).
    fn nvim_synblock_full_clear(block: SynBlockHandle);

    /// Sync-only clear of a synblock (replaces 3 individual sync clearing functions).
    fn nvim_synblock_sync_clear(block: SynBlockHandle);

    /// Free all syntax state stack entries for block.
    fn nvim_syn_stack_free_all(block: SynBlockHandle);

    /// Invalidate the current syntax state.
    fn nvim_syn_invalidate_current_state();

    /// Reset running_syn_inc_tag to 0.
    fn nvim_syn_reset_inc_tag();

    /// Release ownsyntax block: clear it, free it, reset to buf's b_s.
    fn nvim_win_release_synblock(wp: WinHandle);

    // Phase 11 accessors for hashtab keyword operations (Phase 1)

    /// Get number of used entries in hashtab.
    fn nvim_ht_get_used(ht: *const c_void) -> usize;

    /// Get hashtab array size (ht_mask + 1).
    fn nvim_ht_get_array_size(ht: *const c_void) -> usize;

    /// Get HI2KE at array index (null if HASHITEM_EMPTY).
    fn nvim_ht_item_at(ht: *const c_void, idx: usize) -> crate::types::KeyEntryHandle;

    /// Free a keyentry_T and its owned lists.
    fn nvim_ke_free(kp: crate::types::KeyEntryHandle);

    /// hash_clear + hash_init on a hashtab.
    fn nvim_ht_clear_and_init(ht: *mut c_void);

    /// hash_lock a hashtab.
    fn nvim_ht_lock(ht: *mut c_void);

    /// hash_unlock a hashtab.
    fn nvim_ht_unlock(ht: *mut c_void);

    /// Set ke->ke_next.
    fn nvim_ke_set_next(kp: crate::types::KeyEntryHandle, next: crate::types::KeyEntryHandle);

    /// hash_remove at array index idx.
    fn nvim_ht_remove_at(ht: *mut c_void, idx: usize);

    /// Get KE2HIKEY(kp) -- the hi_key string pointer for chain relink.
    fn nvim_ke_get_hikey(kp: crate::types::KeyEntryHandle) -> *mut c_char;

    /// Set ht_array[idx].hi_key = key.
    fn nvim_ht_set_hikey_at(ht: *mut c_void, idx: usize, key: *mut c_char);

    /// Set current_state.ga_itemsize = 0 to mark current state invalid.
    fn nvim_syn_set_current_state_invalid();

    /// Set current_next_list.
    fn nvim_syn_set_current_next_list(list: *mut i16);

    /// Set keepend_level.
    fn nvim_syn_set_keepend_level(level: c_int);
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

    let pp = pat.as_ptr();
    nvim_syn_xfree((*pp).sp_pattern as *mut c_void);
    nvim_syn_vim_regfree((*pp).sp_prog);

    // Only free sp_cont_list, sp_next_list, and sp_syn.cont_in_list for
    // the first start pattern of a group (i == 0 or prev is not SPTYPE_START).
    let free_lists = if i == 0 {
        true
    } else {
        let prev = nvim_synblock_get_pattern(block, i - 1);
        prev.is_null() || i32::from((*prev.as_ptr()).sp_type) != SPTYPE_START
    };

    if free_lists {
        nvim_syn_xfree((*pp).sp_cont_list.cast());
        nvim_syn_xfree((*pp).sp_next_list.cast());
        nvim_syn_xfree((*pp).sp_syn.cont_in_list.cast());
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
    let cp = cluster.as_ptr();
    nvim_syn_xfree((*cp).scl_name as *mut c_void);
    nvim_syn_xfree((*cp).scl_name_u as *mut c_void);
    nvim_syn_xfree((*cp).scl_list.cast());
}

/// Remove one pattern from the buffer's pattern list (compact with memmove).
///
/// # Safety
/// block and idx must be valid; idx < pattern_count.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_remove_pattern(block: SynBlockHandle, idx: c_int) {
    // Decrement fold item count if the pattern has HL_FOLD
    let pat = nvim_synblock_get_pattern(block, idx);
    if !pat.is_null() && ((*pat.as_ptr()).sp_flags & HL_FOLD) != 0 {
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
    let block = nvim_syn_get_curwin_synblock();
    if block.is_null() {
        return;
    }

    // Clear keywords only when not ":syn sync clear group-name"
    if syncing == 0 {
        let ht = nvim_synblock_get_keywtab(block);
        if !ht.is_null() {
            rs_syn_clear_keyword(id, ht);
        }
        let ht_ic = nvim_synblock_get_keywtab_ic(block);
        if !ht_ic.is_null() {
            rs_syn_clear_keyword(id, ht_ic);
        }
    }

    // Clear patterns for "id", iterating from last to first
    let mut idx = nvim_synblock_get_pattern_count(block) - 1;
    while idx >= 0 {
        let pat = nvim_synblock_get_pattern(block, idx);
        if !pat.is_null()
            && i32::from((*pat.as_ptr()).sp_syn.id) == id
            && ((*pat.as_ptr()).sp_syncing as c_int) == syncing
        {
            rs_syn_remove_pattern(block, idx);
        }
        idx -= 1;
    }
}

// =============================================================================
// Phase 3 implementations
// =============================================================================

/// Clear keyword entries with the given id from a hashtable.
/// Implements the logic of C nvim_syn_clear_keyword_in_ht using fine-grained
/// C accessors for the HI2KE/KE2HIKEY pointer arithmetic.
///
/// # Safety
/// ht must be a valid hashtab_T pointer. Must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_clear_keyword(id: c_int, ht: *mut c_void) {
    if ht.is_null() {
        return;
    }
    nvim_ht_lock(ht);
    let array_size = nvim_ht_get_array_size(ht);
    let mut todo = nvim_ht_get_used(ht) as c_int;
    let mut idx: usize = 0;
    while todo > 0 && idx < array_size {
        let head = nvim_ht_item_at(ht, idx);
        if !head.is_null() {
            todo -= 1;
            let mut kp_prev = KeyEntryHandle::null();
            let mut kp = head;
            while !kp.is_null() {
                let kp_next = KeyEntryHandle((*kp.as_ptr()).ke_next);
                if i32::from((*kp.as_ptr()).k_syn.id) == id {
                    if kp_prev.is_null() {
                        if kp_next.is_null() {
                            nvim_ht_remove_at(ht, idx);
                        } else {
                            let key = nvim_ke_get_hikey(kp_next);
                            nvim_ht_set_hikey_at(ht, idx, key);
                        }
                    } else {
                        nvim_ke_set_next(kp_prev, kp_next);
                    }
                    nvim_ke_free(kp);
                } else {
                    kp_prev = kp;
                }
                kp = kp_next;
            }
        }
        idx += 1;
    }
    nvim_ht_unlock(ht);
}

/// Clear a whole keyword table: free all entries and reinitialize.
/// Implements the logic of C nvim_syn_clear_keywtab_ht.
///
/// # Safety
/// ht must be a valid hashtab_T pointer. Must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_clear_keywtab(ht: *mut c_void) {
    if ht.is_null() {
        return;
    }
    let array_size = nvim_ht_get_array_size(ht);
    let mut todo = nvim_ht_get_used(ht) as c_int;
    let mut idx: usize = 0;
    while todo > 0 && idx < array_size {
        let head = nvim_ht_item_at(ht, idx);
        if !head.is_null() {
            todo -= 1;
            let mut entry = head;
            while !entry.is_null() {
                let next = KeyEntryHandle((*entry.as_ptr()).ke_next);
                nvim_ke_free(entry);
                entry = next;
            }
        }
        idx += 1;
    }
    nvim_ht_clear_and_init(ht);
}

/// Mark current_state invalid, clear next_list and keepend_level.
///
/// # Safety
/// Must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_invalidate_current_state() {
    crate::state_ops::rs_syn_clear_current_state();
    nvim_syn_set_current_state_invalid();
    nvim_syn_set_current_next_list(std::ptr::null_mut());
    nvim_syn_set_keepend_level(-1);
}

// =============================================================================
// Phase 2 implementations
// =============================================================================

/// Full teardown of a synblock: free keywords, patterns, clusters, reset flags.
///
/// # Safety
/// block must be a valid synblock_T pointer. Must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syntax_clear(block: SynBlockHandle) {
    if block.is_null() {
        return;
    }

    // Free the syntax patterns (last to first)
    let mut i = nvim_synblock_get_pattern_count(block) - 1;
    while i >= 0 {
        rs_syn_clear_pattern(block, i);
        i -= 1;
    }

    // Free the syntax clusters (last to first)
    let mut i = nvim_synblock_get_cluster_count(block) - 1;
    while i >= 0 {
        rs_syn_clear_cluster(block, i);
        i -= 1;
    }

    // Bulk clear: keytabs, ga arrays, cluster_ids, sync_flags, linecont, syn_isk, scalar flags
    nvim_synblock_full_clear(block);

    // Free the stored states
    nvim_syn_stack_free_all(block);
    nvim_syn_invalidate_current_state();

    // Reset the counter for ":syn include"
    nvim_syn_reset_inc_tag();
}

/// Get rid of ownsyntax for window wp.
///
/// # Safety
/// wp must be a valid win_T pointer. Must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_reset_synblock(wp: WinHandle) {
    if wp.is_null() {
        return;
    }
    nvim_win_release_synblock(wp);
}

/// Clear syncing info for the current window's synblock.
///
/// # Safety
/// Must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syntax_sync_clear() {
    let block = nvim_syn_get_curwin_synblock();
    if block.is_null() {
        return;
    }

    // Free syncing patterns (last to first)
    let mut i = nvim_synblock_get_pattern_count(block) - 1;
    while i >= 0 {
        let pat = nvim_synblock_get_pattern(block, i);
        if !pat.is_null() && (*pat.as_ptr()).sp_syncing {
            rs_syn_remove_pattern(block, i);
        }
        i -= 1;
    }

    // Reset sync flags, linecont, and syn_isk (bulk)
    nvim_synblock_sync_clear(block);

    // Need to recompute all syntax
    nvim_syn_stack_free_all(block);
}
