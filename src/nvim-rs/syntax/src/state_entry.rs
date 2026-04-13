//! State stack entry allocation, removal, and storage.
//!
//! This module migrates from C:
//! - nvim_syn_stack_remove_entry  (linked-list removal + free)
//! - nvim_syn_stack_alloc_entry   (allocation from free list)
//! - nvim_syn_store_state_to_entry (copy current_state to synstate)
//! - nvim_syn_do_stack_realloc    (resize the b_sst_array)

use std::ffi::c_int;

use crate::synblock_struct::{synblock_mut, synblock_ref};
use crate::synstate_struct::{synstate_mut, synstate_ref, SynStateStruct};
use crate::types::{SynBlockHandle, SynStateHandle, SST_FIX_STATES};

// =============================================================================
// FFI declarations
// =============================================================================

extern "C" {
    // Synblock accessors
    fn nvim_syn_get_syn_block() -> SynBlockHandle;

    // SynState setters for store
    fn rs_clear_syn_state(p: SynStateHandle);

    // Phase 11 accessors for rs_syn_store_bufstates Rust implementation
    fn nvim_synstate_ga_init_for_store(sp: SynStateHandle);
    fn nvim_synstate_fill_bufstate_from_curstate(sp: SynStateHandle, i: c_int);

    // Free entry + list operations
    fn rs_syn_stack_free_entry(block: SynBlockHandle, p: SynStateHandle);
    fn rs_syn_stack_cleanup() -> c_int;

    // Array allocation/free
    fn nvim_syn_xcalloc_synstate_array(len: c_int) -> SynStateHandle;
    fn nvim_syn_free_sst_array(ptr: SynStateHandle);

    // syn_time display_tick
    fn nvim_syn_get_display_tick() -> c_int;
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
    let first = SynStateHandle(synblock_ref(block).b_sst_first.cast());
    if first.0 == sp.0 {
        synblock_mut(block).b_sst_first = synstate_ref(sp).sst_next.cast();
    } else {
        let mut p = first;
        while !p.is_null() {
            let pnext = SynStateHandle(synstate_ref(p).sst_next.cast());
            if pnext.0 == sp.0 {
                synstate_mut(p).sst_next = synstate_ref(sp).sst_next;
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
    if synblock_ref(block).b_sst_freecount == 0 {
        rs_syn_stack_cleanup();
    }

    // Still no free items?
    if synblock_ref(block).b_sst_freecount == 0 {
        return SynStateHandle::null();
    }

    // Take the first item from the free list
    let p = SynStateHandle(synblock_ref(block).b_sst_firstfree.cast());
    {
        let b = synblock_mut(block);
        b.b_sst_firstfree = synstate_ref(p).sst_next.cast();
        b.b_sst_freecount -= 1;
    }

    if after.is_null() {
        // Insert at the front of the used list
        synstate_mut(p).sst_next = synblock_ref(block).b_sst_first;
        synblock_mut(block).b_sst_first = p.0.cast();
    } else {
        // Insert after the given entry
        synstate_mut(p).sst_next = synstate_ref(after).sst_next;
        synstate_mut(after).sst_next = p.0.cast();
    }

    synstate_mut(p).sst_stacksize = 0;
    synstate_mut(p).sst_lnum = lnum;
    p
}

/// Fill bufstate array in a synstate entry from the current state stack.
///
/// Replaces C `nvim_syn_store_bufstates`. Handles both fixed-array
/// (stacksize <= SST_FIX_STATES) and growarray paths.
///
/// # Safety
/// Accesses C global current_state; must be called from main thread.
/// Must be called after nvim_synstate_set_stacksize has been set on sp.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_store_bufstates(sp: SynStateHandle) {
    if sp.is_null() {
        return;
    }
    let stacksize = synstate_ref(sp).sst_stacksize;
    let sst_fix_states = SST_FIX_STATES;
    if stacksize > sst_fix_states {
        nvim_synstate_ga_init_for_store(sp);
    }
    for i in 0..stacksize {
        nvim_synstate_fill_bufstate_from_curstate(sp, i);
    }
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
    rs_clear_syn_state(sp);

    let stacksize = crate::statics::CURRENT_STATE.ga_len;
    synstate_mut(sp).sst_stacksize = stacksize;

    // Fill bufstate array (handles union fixed/growarray split)
    rs_syn_store_bufstates(sp);

    // Copy next_flags, next_list, tick, change_lnum
    synstate_mut(sp).sst_next_flags = crate::statics::CURRENT_NEXT_FLAGS;
    synstate_mut(sp).sst_next_list = crate::statics::CURRENT_NEXT_LIST;
    synstate_mut(sp).sst_tick = nvim_syn_get_display_tick() as u64;
    synstate_mut(sp).sst_change_lnum = 0;
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

    // Get the base pointer of the new array as *mut SynStateStruct
    let base: *mut SynStateStruct = sstp.0.cast();

    // Copy existing used entries from the linked list to the new array
    let mut used_count = 0;
    let mut from = SynStateHandle(synblock_ref(block).b_sst_first.cast());
    while !from.is_null() && used_count < len {
        // Destination = base + used_count
        let to_ptr = base.add(used_count as usize);
        // Copy the entire struct (memcpy equivalent)
        std::ptr::copy_nonoverlapping(from.0.cast::<SynStateStruct>(), to_ptr, 1);
        // Set sst_next to point to the next slot (or null for last)
        (*to_ptr).sst_next = base.add((used_count + 1) as usize);
        from = SynStateHandle(synstate_ref(from).sst_next.cast());
        used_count += 1;
    }

    // Fix the last used entry's sst_next to be null
    if used_count > 0 {
        (*base.add((used_count - 1) as usize)).sst_next = std::ptr::null_mut();
        {
            let b = synblock_mut(block);
            b.b_sst_first = base.cast();
            b.b_sst_freecount = len - used_count;
        }
    } else {
        {
            let b = synblock_mut(block);
            b.b_sst_first = std::ptr::null_mut();
            b.b_sst_freecount = len;
        }
    }

    // Build the free list starting at base[used_count]
    synblock_mut(block).b_sst_firstfree = base.add(used_count as usize).cast();

    // Chain free entries together
    for i in used_count..(len - 1) {
        (*base.add(i as usize)).sst_next = base.add((i + 1) as usize);
    }
    // Terminate the free list
    if used_count < len {
        (*base.add((len - 1) as usize)).sst_next = std::ptr::null_mut();
    }

    // Free the old array
    let old_array = SynStateHandle(synblock_ref(block).b_sst_array.cast());
    nvim_syn_free_sst_array(old_array);

    // Install the new array
    {
        let b = synblock_mut(block);
        b.b_sst_array = sstp.0.cast();
        b.b_sst_len = len;
    }
}
