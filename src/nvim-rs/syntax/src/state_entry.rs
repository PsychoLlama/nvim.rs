//! State stack entry allocation, removal, and storage.
//!
//! This module migrates from C:
//! - nvim_syn_stack_remove_entry  (linked-list removal + free)
//! - nvim_syn_stack_alloc_entry   (allocation from free list)
//! - nvim_syn_store_state_to_entry (copy current_state to synstate)
//! - nvim_syn_do_stack_realloc    (resize the b_sst_array)

use std::ffi::c_int;

use crate::types::{IdListHandle, SynBlockHandle, SynStateHandle};

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    // Synblock accessors
    fn nvim_syn_get_syn_block() -> SynBlockHandle;
    fn nvim_synblock_get_sst_first(block: SynBlockHandle) -> SynStateHandle;
    fn nvim_synblock_set_sst_first(block: SynBlockHandle, ptr: SynStateHandle);
    fn nvim_synblock_get_sst_firstfree(block: SynBlockHandle) -> SynStateHandle;
    fn nvim_synblock_set_sst_firstfree(block: SynBlockHandle, ptr: SynStateHandle);
    fn nvim_synblock_get_sst_freecount(block: SynBlockHandle) -> c_int;
    fn nvim_synblock_set_sst_freecount(block: SynBlockHandle, count: c_int);
    fn nvim_synblock_set_sst_array(block: SynBlockHandle, ptr: SynStateHandle, len: c_int);

    // SynState navigation
    fn nvim_synstate_get_next(state: SynStateHandle) -> SynStateHandle;
    fn nvim_synstate_set_next(state: SynStateHandle, next: SynStateHandle);
    fn nvim_synstate_set_stacksize(state: SynStateHandle, size: c_int);
    fn nvim_synstate_set_sst_lnum(state: SynStateHandle, lnum: c_int);

    // SynState setters for store
    fn nvim_syn_do_clear_syn_state(p: SynStateHandle);
    fn nvim_synstate_set_sst_next_flags(state: SynStateHandle, flags: c_int);
    fn nvim_synstate_set_sst_next_list(state: SynStateHandle, list: IdListHandle);
    fn nvim_synstate_set_sst_change_lnum(state: SynStateHandle, lnum: c_int);
    fn nvim_synstate_set_tick_to_display(state: SynStateHandle);

    // Stack size from current_state
    fn nvim_syn_get_current_state_len() -> c_int;
    fn nvim_syn_get_current_next_flags() -> c_int;
    fn nvim_syn_get_current_next_list() -> IdListHandle;

    // Bufstate fill (handles union sst_stack vs sst_ga)
    fn nvim_syn_store_bufstates(sp: SynStateHandle);

    // Free entry + list operations
    fn rs_syn_stack_free_entry(block: SynBlockHandle, p: SynStateHandle);
    fn rs_syn_stack_cleanup() -> c_int;

    // Array allocation/free
    fn nvim_syn_xcalloc_synstate_array(len: c_int) -> SynStateHandle;
    fn nvim_syn_free_sst_array(ptr: SynStateHandle);
    fn nvim_synblock_get_sst_array_ptr(block: SynBlockHandle) -> SynStateHandle;
}

// =============================================================================
// Phase 9 Rust implementations
// =============================================================================

/// Remove a state entry from the used list and move to free list.
///
/// Replaces C `nvim_syn_stack_remove_entry`.
///
/// # Safety
/// Accesses C global syn_block state; must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_stack_remove_entry(sp: SynStateHandle) {
    if sp.is_null() {
        return;
    }
    let block = nvim_syn_get_syn_block();
    if block.is_null() {
        return;
    }
    let first = nvim_synblock_get_sst_first(block);
    if first.0 == sp.0 {
        nvim_synblock_set_sst_first(block, nvim_synstate_get_next(sp));
    } else {
        let mut p = first;
        while !p.is_null() {
            let pnext = nvim_synstate_get_next(p);
            if pnext.0 == sp.0 {
                nvim_synstate_set_next(p, nvim_synstate_get_next(sp));
                break;
            }
            p = pnext;
        }
    }
    rs_syn_stack_free_entry(block, sp);
}

/// Allocate a new state entry for the given line.
/// Inserts it after `after` (or at the front if `after` is null).
/// Returns null if no free entries are available.
///
/// Replaces C `nvim_syn_stack_alloc_entry`.
///
/// # Safety
/// Accesses C global syn_block state; must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_stack_alloc_entry(
    lnum: c_int,
    after: SynStateHandle,
) -> SynStateHandle {
    let block = nvim_syn_get_syn_block();
    if block.is_null() {
        return SynStateHandle::null();
    }

    // If no free items, try to cleanup first
    if nvim_synblock_get_sst_freecount(block) == 0 {
        rs_syn_stack_cleanup();
    }

    // Still no free items?
    if nvim_synblock_get_sst_freecount(block) == 0 {
        return SynStateHandle::null();
    }

    // Take the first item from the free list
    let p = nvim_synblock_get_sst_firstfree(block);
    nvim_synblock_set_sst_firstfree(block, nvim_synstate_get_next(p));
    nvim_synblock_set_sst_freecount(block, nvim_synblock_get_sst_freecount(block) - 1);

    if after.is_null() {
        // Insert at the front of the used list
        nvim_synstate_set_next(p, nvim_synblock_get_sst_first(block));
        nvim_synblock_set_sst_first(block, p);
    } else {
        // Insert after the given entry
        nvim_synstate_set_next(p, nvim_synstate_get_next(after));
        nvim_synstate_set_next(after, p);
    }

    nvim_synstate_set_stacksize(p, 0);
    nvim_synstate_set_sst_lnum(p, lnum);
    p
}

/// Store the current state into a synstate entry.
///
/// Replaces C `nvim_syn_store_state_to_entry`.
///
/// # Safety
/// Accesses C global syn_block state; must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_store_state_to_entry(sp: SynStateHandle) {
    if sp.is_null() {
        return;
    }

    // Clear any existing state data
    nvim_syn_do_clear_syn_state(sp);

    let stacksize = nvim_syn_get_current_state_len();
    nvim_synstate_set_stacksize(sp, stacksize);

    // Fill bufstate array (handles union fixed/growarray split)
    nvim_syn_store_bufstates(sp);

    // Copy next_flags, next_list, tick, change_lnum
    nvim_synstate_set_sst_next_flags(sp, nvim_syn_get_current_next_flags());
    nvim_synstate_set_sst_next_list(sp, nvim_syn_get_current_next_list());
    nvim_synstate_set_tick_to_display(sp);
    nvim_synstate_set_sst_change_lnum(sp, 0);
}

/// Reallocate b_sst_array, copy existing entries from used list,
/// and rebuild the free list.
///
/// Replaces C `nvim_syn_do_stack_realloc`.
///
/// # Safety
/// Accesses C global syn_block state; must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_do_stack_realloc(len: c_int) {
    assert!(len >= 0);

    let block = nvim_syn_get_syn_block();
    if block.is_null() {
        return;
    }

    // Allocate new zeroed array
    let sstp = nvim_syn_xcalloc_synstate_array(len);

    // Copy existing used entries from the linked list to the new array
    let mut used_count = 0;
    let mut from = nvim_synblock_get_sst_first(block);
    while !from.is_null() && used_count < len {
        // Destination = sstp + used_count
        let to = nvim_syn_sst_array_at(sstp, used_count);
        nvim_syn_sst_copy_entry(to, from);
        // Set sst_next to point to the next slot (or null for last)
        nvim_synstate_set_next(to, nvim_syn_sst_array_at(sstp, used_count + 1));
        from = nvim_synstate_get_next(from);
        used_count += 1;
    }

    // Fix the last used entry's sst_next to be null
    if used_count > 0 {
        let last_used = nvim_syn_sst_array_at(sstp, used_count - 1);
        nvim_synstate_set_next(last_used, SynStateHandle::null());
        nvim_synblock_set_sst_first(block, sstp);
        let free_count = len - used_count;
        nvim_synblock_set_sst_freecount(block, free_count);
    } else {
        nvim_synblock_set_sst_first(block, SynStateHandle::null());
        nvim_synblock_set_sst_freecount(block, len);
    }

    // Build the free list starting at sstp[used_count]
    let firstfree = nvim_syn_sst_array_at(sstp, used_count);
    nvim_synblock_set_sst_firstfree(block, firstfree);

    // Chain free entries together
    for i in used_count..(len - 1) {
        let entry = nvim_syn_sst_array_at(sstp, i);
        nvim_synstate_set_next(entry, nvim_syn_sst_array_at(sstp, i + 1));
    }
    // Terminate the free list
    if used_count < len {
        let last_free = nvim_syn_sst_array_at(sstp, len - 1);
        nvim_synstate_set_next(last_free, SynStateHandle::null());
    }

    // Free the old array
    let old_array = nvim_synblock_get_sst_array_ptr(block);
    nvim_syn_free_sst_array(old_array);

    // Install the new array
    nvim_synblock_set_sst_array(block, sstp, len);
}

// Helper: get pointer to synstate entry at index in array (used in do_stack_realloc)
extern "C" {
    fn nvim_syn_sst_array_at(array: SynStateHandle, idx: c_int) -> SynStateHandle;
    fn nvim_syn_sst_copy_entry(dst: SynStateHandle, src: SynStateHandle);
}
