//! Syntax state caching infrastructure.
//!
//! This module handles line-level state caching for efficient syntax highlighting:
//! - State stack allocation and management (b_sst_array)
//! - Cache lookup and storage operations
//! - State comparison for cache validation
//! - Cache invalidation logic

use std::ffi::c_int;

use crate::types::{
    BufStateHandle, ExtMatchHandle, IdListHandle, SynBlockHandle, SynStateHandle, HL_KEEPEND,
};

// =============================================================================
// FFI declarations for cache operations
// =============================================================================

extern "C" {
    // State stack management
    fn nvim_syn_stack_find_entry(lnum: c_int) -> SynStateHandle;
    fn nvim_syn_stack_remove_entry(sp: SynStateHandle);
    fn nvim_syn_stack_alloc_entry(lnum: c_int, after: SynStateHandle) -> SynStateHandle;
    fn nvim_syn_store_state_to_entry(sp: SynStateHandle);
    fn nvim_syn_stack_alloc();
    fn nvim_syn_stack_find_entry_ptr(lnum: c_int) -> SynStateHandle;

    // State accessors for comparison
    fn nvim_synstate_get_lnum(state: SynStateHandle) -> c_int;
    fn nvim_synstate_get_stacksize(state: SynStateHandle) -> c_int;
    fn nvim_synstate_get_next_flags(state: SynStateHandle) -> c_int;
    fn nvim_synstate_get_next_list(state: SynStateHandle) -> IdListHandle;
    fn nvim_synstate_get_bufstate(state: SynStateHandle, idx: c_int) -> BufStateHandle;
    fn nvim_synstate_set_change_lnum(state: SynStateHandle, lnum: c_int);

    // Bufstate accessors
    fn nvim_bufstate_get_idx(bs: BufStateHandle) -> c_int;
    fn nvim_bufstate_get_flags(bs: BufStateHandle) -> c_int;
    fn nvim_bufstate_get_seqnr(bs: BufStateHandle) -> c_int;
    fn nvim_bufstate_get_cchar(bs: BufStateHandle) -> c_int;
    fn nvim_bufstate_get_extmatch(bs: BufStateHandle) -> ExtMatchHandle;

    // Current state accessors
    fn nvim_syn_get_current_lnum() -> c_int;
    fn nvim_syn_get_current_state_len() -> c_int;
    fn nvim_syn_get_current_next_list() -> IdListHandle;
    fn nvim_syn_state_item_spans_line(idx: c_int, lnum: c_int) -> c_int;
    fn nvim_syn_set_state_stored(stored: c_int);
    fn nvim_syn_clear_current_state();
    fn nvim_syn_validate_current_state();
    fn nvim_syn_set_keepend_level(level: c_int);
    fn nvim_syn_grow_current_state(size: c_int);
    fn nvim_syn_set_current_state_len(len: c_int);
    fn nvim_syn_set_current_next_list(list: IdListHandle);
    fn nvim_syn_set_current_next_flags(flags: c_int);
    fn nvim_syn_set_current_lnum(lnum: c_int);

    // State item operations
    fn nvim_syn_set_cur_state_item(
        idx: c_int,
        si_idx: c_int,
        si_flags: c_int,
        si_seqnr: c_int,
        si_cchar: c_int,
        em: ExtMatchHandle,
    );
    fn nvim_syn_update_si_attr(idx: c_int);
    fn nvim_syn_get_cur_state(idx: c_int) -> crate::types::StateItemHandle;

    // Stateitem accessors for comparison
    fn nvim_stateitem_get_idx(item: crate::types::StateItemHandle) -> c_int;
    fn nvim_stateitem_get_flags(item: crate::types::StateItemHandle) -> c_int;
    fn nvim_stateitem_get_seqnr(item: crate::types::StateItemHandle) -> c_int;
    fn nvim_stateitem_get_cchar(item: crate::types::StateItemHandle) -> c_int;
    fn nvim_stateitem_get_extmatch(item: crate::types::StateItemHandle) -> ExtMatchHandle;

    // Extmatch comparison
    fn nvim_syn_extmatch_equal(a: ExtMatchHandle, b: ExtMatchHandle) -> c_int;
    fn nvim_syn_get_nsubexp() -> c_int;
    fn nvim_syn_extmatch_strings_equal(
        a: ExtMatchHandle,
        b: ExtMatchHandle,
        idx: c_int,
        count: c_int,
    ) -> c_int;

    // Synblock accessors
    fn nvim_synblock_get_sst_first(block: SynBlockHandle) -> SynStateHandle;
    fn nvim_synblock_get_sst_len(block: SynBlockHandle) -> c_int;
    fn nvim_synblock_get_sst_freecount(block: SynBlockHandle) -> c_int;
    fn nvim_synblock_has_sst_array(block: SynBlockHandle) -> c_int;

    // Global state accessors
    fn nvim_syn_get_sst_array() -> *mut std::ffi::c_void;
    fn nvim_syn_get_sst_first() -> SynStateHandle;
    fn nvim_syn_get_sst_len() -> c_int;
}

// =============================================================================
// Cache lookup and storage
// =============================================================================

/// Find a synstate entry for the given line number.
/// Returns the entry at or before the line, or null if none found.
#[must_use]
pub fn stack_find_entry(lnum: i32) -> SynStateHandle {
    unsafe { nvim_syn_stack_find_entry(lnum) }
}

/// Find a synstate entry for the given line number (alternate interface).
#[must_use]
pub fn stack_find_entry_ptr(lnum: i32) -> SynStateHandle {
    unsafe { nvim_syn_stack_find_entry_ptr(lnum) }
}

/// Remove a synstate entry from the cache.
pub fn stack_remove_entry(sp: SynStateHandle) {
    if !sp.is_null() {
        unsafe { nvim_syn_stack_remove_entry(sp) }
    }
}

/// Allocate a new synstate entry at the given line number.
/// The entry is inserted after the given state (or at the beginning if null).
#[must_use]
pub fn stack_alloc_entry(lnum: i32, after: SynStateHandle) -> SynStateHandle {
    unsafe { nvim_syn_stack_alloc_entry(lnum, after) }
}

/// Store the current state to a synstate entry.
pub fn store_state_to_entry(sp: SynStateHandle) {
    if !sp.is_null() {
        unsafe { nvim_syn_store_state_to_entry(sp) }
    }
}

/// Allocate the syntax stack array (b_sst_array).
pub fn stack_alloc() {
    unsafe { nvim_syn_stack_alloc() }
}

// =============================================================================
// State caching high-level operations
// =============================================================================

/// Try saving the current state in the state cache.
/// The current state must be valid for the start of the current_lnum line!
/// Returns the synstate entry (or null if not stored).
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[must_use]
pub unsafe fn store_current_state() -> SynStateHandle {
    let lnum = nvim_syn_get_current_lnum();
    let state_len = nvim_syn_get_current_state_len();

    // Find existing entry at or before current line
    let sp = nvim_syn_stack_find_entry(lnum);

    // Check if current state contains items that span across lines
    // If so, we can't use this state - it's not valid for line boundaries
    let mut has_spanning_item = false;
    for i in (0..state_len).rev() {
        if nvim_syn_state_item_spans_line(i, lnum) != 0 {
            has_spanning_item = true;
            break;
        }
    }

    if has_spanning_item {
        // Current state spans lines, can't store it
        // If there was an existing entry at this line, remove it
        if !sp.is_null() {
            nvim_syn_stack_remove_entry(sp);
        }
        nvim_syn_set_state_stored(1);
        return SynStateHandle::null();
    }

    // Determine if we need to allocate a new entry
    let entry = if sp.is_null() || nvim_synstate_get_lnum(sp) != lnum {
        // Need to allocate a new entry
        nvim_syn_stack_alloc_entry(lnum, sp)
    } else {
        // Reuse existing entry
        sp
    };

    if !entry.is_null() {
        // Store current state to the entry
        nvim_syn_store_state_to_entry(entry);
    }

    nvim_syn_set_state_stored(1);
    entry
}

/// Load a state from a synstate entry into the current state.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
pub unsafe fn load_current_state(from: SynStateHandle) {
    if from.is_null() {
        return;
    }

    // Clear and validate current state
    nvim_syn_clear_current_state();
    nvim_syn_validate_current_state();
    nvim_syn_set_keepend_level(-1);

    let stacksize = nvim_synstate_get_stacksize(from);
    if stacksize > 0 {
        // Grow current state array
        nvim_syn_grow_current_state(stacksize);
        nvim_syn_set_current_state_len(stacksize);

        // Copy each state item
        let mut keepend_level = -1;
        for i in 0..stacksize {
            let bs = nvim_synstate_get_bufstate(from, i);
            if bs.is_null() {
                continue;
            }

            let bs_idx = nvim_bufstate_get_idx(bs);
            let bs_flags = nvim_bufstate_get_flags(bs);
            let bs_seqnr = nvim_bufstate_get_seqnr(bs);
            let bs_cchar = nvim_bufstate_get_cchar(bs);
            let extmatch = nvim_bufstate_get_extmatch(bs);

            // Set the state item (this also sets si_next_list based on pattern)
            nvim_syn_set_cur_state_item(i, bs_idx, bs_flags, bs_seqnr, bs_cchar, extmatch);

            // Track keepend level
            if keepend_level < 0 && (bs_flags & HL_KEEPEND) != 0 {
                keepend_level = i;
            }

            // Update attributes for this item
            nvim_syn_update_si_attr(i);
        }

        nvim_syn_set_keepend_level(keepend_level);
    }

    // Copy next_list and next_flags from saved state
    let next_list = nvim_synstate_get_next_list(from);
    nvim_syn_set_current_next_list(next_list);
    nvim_syn_set_current_next_flags(nvim_synstate_get_next_flags(from));
    nvim_syn_set_current_lnum(nvim_synstate_get_lnum(from));
}

// =============================================================================
// State comparison for cache validation
// =============================================================================

/// Compare saved state stack with the current state.
/// Returns true if they are equal.
///
/// # Safety
/// This function accesses C global state and must be called from the main thread.
#[must_use]
pub unsafe fn syn_stack_equal(sp: SynStateHandle) -> bool {
    if sp.is_null() {
        return false;
    }

    let sp_stacksize = nvim_synstate_get_stacksize(sp);
    let current_len = nvim_syn_get_current_state_len();

    // Quick check: stack sizes must match
    if sp_stacksize != current_len {
        return false;
    }

    // Quick check: next_list pointers must match
    let sp_next_list = nvim_synstate_get_next_list(sp);
    let cur_next_list = nvim_syn_get_current_next_list();
    if sp_next_list.0 != cur_next_list.0 {
        return false;
    }

    // Compare each state item
    let nsubexp = nvim_syn_get_nsubexp();
    for i in (0..current_len).rev() {
        let bs = nvim_synstate_get_bufstate(sp, i);
        if bs.is_null() {
            return false;
        }

        let cur_si = nvim_syn_get_cur_state(i);
        if cur_si.is_null() {
            return false;
        }

        // Compare basic fields
        if nvim_bufstate_get_idx(bs) != nvim_stateitem_get_idx(cur_si) {
            return false;
        }
        if nvim_bufstate_get_flags(bs) != nvim_stateitem_get_flags(cur_si) {
            return false;
        }
        if nvim_bufstate_get_seqnr(bs) != nvim_stateitem_get_seqnr(cur_si) {
            return false;
        }
        if nvim_bufstate_get_cchar(bs) != nvim_stateitem_get_cchar(cur_si) {
            return false;
        }

        // Compare external matches
        let bs_extmatch = nvim_bufstate_get_extmatch(bs);
        let si_extmatch = nvim_stateitem_get_extmatch(cur_si);

        // If both are null, they're equal
        if bs_extmatch.is_null() && si_extmatch.is_null() {
            continue;
        }

        // If one is null and the other isn't, check for empty strings
        if bs_extmatch.is_null() || si_extmatch.is_null() {
            // Check if the non-null one has any actual content
            if nvim_syn_extmatch_strings_equal(bs_extmatch, si_extmatch, 0, nsubexp) == 0 {
                return false;
            }
            continue;
        }

        // Both non-null: compare using the C comparison function
        if nvim_syn_extmatch_equal(bs_extmatch, si_extmatch) == 0 {
            return false;
        }
    }

    true
}

// =============================================================================
// Cache invalidation
// =============================================================================

/// Invalidate all cached states after the given line number.
/// This is called when a change is made that might affect highlighting.
pub fn invalidate_states_after(block: SynBlockHandle, lnum: i32) {
    if block.is_null() {
        return;
    }

    let mut state = unsafe { nvim_synblock_get_sst_first(block) };
    while !state.is_null() {
        let state_lnum = unsafe { nvim_synstate_get_lnum(state) };
        if state_lnum >= lnum {
            // Mark this state as invalid
            unsafe { nvim_synstate_set_change_lnum(state, lnum) }
        }
        state = crate::state::synstate_next(state);
    }
}

// =============================================================================
// Cache statistics
// =============================================================================

/// Get the first used state in the cache
#[must_use]
pub fn get_sst_first() -> SynStateHandle {
    unsafe { nvim_syn_get_sst_first() }
}

/// Get the state array length
#[must_use]
pub fn get_sst_len() -> i32 {
    unsafe { nvim_syn_get_sst_len() }
}

/// Get the raw state array pointer
#[must_use]
pub fn get_sst_array() -> *mut std::ffi::c_void {
    unsafe { nvim_syn_get_sst_array() }
}

/// Check if a synblock has a state array allocated
#[must_use]
pub fn synblock_has_state_array(block: SynBlockHandle) -> bool {
    if block.is_null() {
        return false;
    }
    unsafe { nvim_synblock_has_sst_array(block) != 0 }
}

/// Get the state array length for a synblock
#[must_use]
pub fn synblock_state_array_len(block: SynBlockHandle) -> i32 {
    if block.is_null() {
        return 0;
    }
    unsafe { nvim_synblock_get_sst_len(block) }
}

/// Get the number of free entries in the state array
#[must_use]
pub fn synblock_free_state_count(block: SynBlockHandle) -> i32 {
    if block.is_null() {
        return 0;
    }
    unsafe { nvim_synblock_get_sst_freecount(block) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_null_handle_checks() {
        // These tests verify null handle checks work correctly
        // Note: We can only test null checking, not functions that call extern FFI
        let null_state = SynStateHandle::null();
        let null_block = SynBlockHandle(std::ptr::null_mut());

        assert!(null_state.is_null());
        assert!(null_block.is_null());

        // Non-null handle creation (for testing purposes only)
        let fake_ptr = std::ptr::dangling_mut::<std::ffi::c_void>();
        let non_null_block = SynBlockHandle(fake_ptr);
        assert!(!non_null_block.is_null());
    }
}
