//! Syntax state caching infrastructure.
//!
//! This module handles line-level state caching for efficient syntax highlighting:
//! - State stack allocation and management (b_sst_array)
//! - Cache lookup and storage operations
//! - State comparison for cache validation
//! - Cache invalidation logic

use std::ffi::{c_int, c_void};

use crate::ffi_types::{BufState, StateItem};
use crate::types::{
    BufHandle, BufStateHandle, ExtMatchHandle, IdListHandle, SynBlockHandle, SynStateHandle,
    HL_KEEPEND, SST_DIST, SST_MAX_ENTRIES, SST_MIN_ENTRIES,
};

// =============================================================================
// FFI declarations for cache operations
// =============================================================================

extern "C" {
    // (nvim_syn_stack_find_entry/stack_alloc deleted: call Rust directly)

    // State accessors for comparison
    fn nvim_synstate_get_lnum(state: SynStateHandle) -> c_int;
    fn nvim_synstate_get_stacksize(state: SynStateHandle) -> c_int;
    fn nvim_synstate_get_next_flags(state: SynStateHandle) -> c_int;
    fn nvim_synstate_get_next_list(state: SynStateHandle) -> IdListHandle;
    fn nvim_synstate_get_bufstate(state: SynStateHandle, idx: c_int) -> BufStateHandle;
    fn nvim_synstate_set_change_lnum(state: SynStateHandle, lnum: c_int);

    // Current state accessors
    #[link_name = "rs_validate_current_state"]
    fn nvim_syn_validate_current_state();
    fn nvim_syn_update_si_attr(idx: c_int);

    // Synblock accessors
    fn nvim_synblock_get_sst_first(block: SynBlockHandle) -> SynStateHandle;
    fn nvim_synblock_get_sst_len(block: SynBlockHandle) -> c_int;
    fn nvim_synblock_get_sst_freecount(block: SynBlockHandle) -> c_int;
    fn nvim_synblock_has_sst_array(block: SynBlockHandle) -> c_int;

    // Global state accessors
    fn nvim_syn_get_sst_array() -> *mut c_void;
    fn nvim_syn_get_sst_first() -> SynStateHandle;
    fn nvim_syn_get_sst_len() -> c_int;

    // -------------------------------------------------------------------------
    // Phase 8 accessors: state stack cache management
    // -------------------------------------------------------------------------

    // synstate_T setters / navigation
    fn nvim_synstate_get_next(state: SynStateHandle) -> SynStateHandle;
    fn nvim_synstate_set_next(state: SynStateHandle, next: SynStateHandle);
    fn nvim_synstate_get_tick(state: SynStateHandle) -> c_int;
    fn nvim_synstate_set_lnum(state: SynStateHandle, lnum: c_int);
    fn nvim_synstate_get_change_lnum(state: SynStateHandle) -> c_int;

    // synblock_T setters
    fn nvim_synblock_set_sst_first(block: SynBlockHandle, ptr: SynStateHandle);
    fn nvim_synblock_set_sst_firstfree(block: SynBlockHandle, ptr: SynStateHandle);
    fn nvim_synblock_set_sst_freecount(block: SynBlockHandle, count: c_int);
    fn nvim_synblock_set_sst_array(block: SynBlockHandle, ptr: SynStateHandle, len: c_int);
    fn nvim_synblock_get_sst_firstfree(block: SynBlockHandle) -> SynStateHandle;
    fn nvim_synblock_get_sst_lasttick(block: SynBlockHandle) -> c_int;
    fn nvim_synblock_get_sync_linebreaks(block: SynBlockHandle) -> c_int;
    fn nvim_synblock_get_sst_array_ptr(block: SynBlockHandle) -> SynStateHandle;

    // clear_syn_state (Rust implementation)
    fn rs_clear_syn_state(p: SynStateHandle);

    // Array allocation/free
    fn nvim_syn_xcalloc_synstate_array(len: c_int) -> SynStateHandle;
    fn nvim_syn_free_sst_array(ptr: SynStateHandle);

    // Global syn_block handle
    fn nvim_syn_get_syn_block() -> SynBlockHandle;

    // syn_buf line count
    fn nvim_syn_buf_get_ml_line_count() -> c_int;

    // Rows
    fn nvim_syn_get_rows() -> c_int;

    // Fold update for windows in tab
    fn nvim_syn_fold_update_for_block(block: SynBlockHandle);
    fn nvim_syn_apply_changes_for_windows(buf: BufHandle);

    // buf->b_s accessor
    fn nvim_buf_get_b_s(buf: BufHandle) -> SynBlockHandle;

    // buf modification info
    fn nvim_buf_get_mod_top(buf: BufHandle) -> c_int;
    fn nvim_buf_get_mod_bot(buf: BufHandle) -> c_int;
    fn nvim_buf_get_mod_xlines(buf: BufHandle) -> c_int;

    // nvim_synblock_get_sync_linebreaks already declared above
}

// =============================================================================
// Phase 8: State stack cache management (replaces C implementations)
// =============================================================================

/// Find an entry in the list of state stacks at or before "lnum".
/// Returns null when there is no entry or the first entry is after "lnum".
///
/// Replaces static C `syn_stack_find_entry`.
///
/// # Safety
/// Accesses C global syn_block state; must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_stack_find_entry(lnum: c_int) -> SynStateHandle {
    let block = nvim_syn_get_syn_block();
    if block.is_null() {
        return SynStateHandle::null();
    }
    let mut prev = SynStateHandle::null();
    let mut p = nvim_synblock_get_sst_first(block);
    while !p.is_null() {
        let p_lnum = nvim_synstate_get_lnum(p);
        if p_lnum == lnum {
            return p;
        }
        if p_lnum > lnum {
            break;
        }
        prev = p;
        p = nvim_synstate_get_next(p);
    }
    prev
}

/// Free one synstate entry and move it to the free list.
///
/// Replaces static C `syn_stack_free_entry`.
///
/// # Safety
/// Accesses C global syn_block state; must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_stack_free_entry(block: SynBlockHandle, p: SynStateHandle) {
    if block.is_null() || p.is_null() {
        return;
    }
    rs_clear_syn_state(p);
    let firstfree = nvim_synblock_get_sst_firstfree(block);
    nvim_synstate_set_next(p, firstfree);
    nvim_synblock_set_sst_firstfree(block, p);
    let count = nvim_synblock_get_sst_freecount(block);
    nvim_synblock_set_sst_freecount(block, count + 1);
}

/// Free all entries in a synblock's state array.
///
/// Replaces static C `syn_stack_free_block`.
///
/// # Safety
/// Accesses C global state; must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_stack_free_block(block: SynBlockHandle) {
    if block.is_null() {
        return;
    }
    if nvim_synblock_has_sst_array(block) == 0 {
        return;
    }
    // Clear all used entries
    let mut p = nvim_synblock_get_sst_first(block);
    while !p.is_null() {
        let next = nvim_synstate_get_next(p);
        rs_clear_syn_state(p);
        p = next;
    }
    // Free the backing array and reset fields
    let arr = nvim_synblock_get_sst_array_ptr(block);
    nvim_syn_free_sst_array(arr);
    nvim_synblock_set_sst_array(block, SynStateHandle::null(), 0);
    nvim_synblock_set_sst_first(block, SynStateHandle::null());
    nvim_synblock_set_sst_freecount(block, 0);
}

/// Free b_sst_array[] for the given synblock.
/// Also updates folds for affected windows via C helper.
///
/// Replaces public C `syn_stack_free_all`.
///
/// # Safety
/// Accesses C global state; must be called from main thread.
#[export_name = "syn_stack_free_all"]
pub unsafe extern "C" fn rs_syn_stack_free_all(block: SynBlockHandle) {
    rs_syn_stack_free_block(block);
    // FOR_ALL_WINDOWS_IN_TAB macro stays in C
    nvim_syn_fold_update_for_block(block);
}

/// Reduce the number of entries in the state stack for syn_buf.
/// Returns 1 if at least one entry was freed, 0 otherwise.
///
/// Replaces static C `syn_stack_cleanup`.
///
/// # Safety
/// Accesses C global state; must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_stack_cleanup() -> c_int {
    let block = nvim_syn_get_syn_block();
    if block.is_null() {
        return 0;
    }
    let first = nvim_synblock_get_sst_first(block);
    if first.is_null() {
        return 0;
    }

    let sst_len = nvim_synblock_get_sst_len(block);
    let rows = nvim_syn_get_rows();

    // Compute normal distance between non-displayed entries.
    let dist = if sst_len <= rows {
        999999
    } else {
        let line_count = nvim_syn_buf_get_ml_line_count();
        line_count / (sst_len - rows) + 1
    };

    // Find the "tick" for the oldest entry that can be removed.
    let lasttick = nvim_synblock_get_sst_lasttick(block);
    let mut tick = lasttick;
    let mut above = false;

    let mut prev = first;
    let mut p = nvim_synstate_get_next(first);
    while !p.is_null() {
        let prev_lnum = nvim_synstate_get_lnum(prev);
        let p_lnum = nvim_synstate_get_lnum(p);
        if prev_lnum + dist > p_lnum {
            let p_tick = nvim_synstate_get_tick(p);
            if p_tick > lasttick {
                if !above || p_tick < tick {
                    tick = p_tick;
                }
                above = true;
            } else if !above && p_tick < tick {
                tick = p_tick;
            }
        }
        prev = p;
        p = nvim_synstate_get_next(p);
    }

    // Remove entries at the oldest tick that are too close together.
    let mut retval = 0;
    prev = first;
    p = nvim_synstate_get_next(first);
    while !p.is_null() {
        let prev_lnum = nvim_synstate_get_lnum(prev);
        let p_lnum = nvim_synstate_get_lnum(p);
        let p_tick = nvim_synstate_get_tick(p);
        let next = nvim_synstate_get_next(p);
        if p_tick == tick && prev_lnum + dist > p_lnum {
            // Move this entry from used list to free list
            nvim_synstate_set_next(prev, next);
            rs_syn_stack_free_entry(block, p);
            // prev stays the same (we removed p, not prev)
            retval = 1;
        } else {
            prev = p;
        }
        p = next;
    }
    retval
}

/// Allocate/resize the b_sst_array for syn_buf when needed.
///
/// Replaces static C `syn_stack_alloc`.
///
/// # Safety
/// Accesses C global state; must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_stack_alloc() {
    let block = nvim_syn_get_syn_block();
    if block.is_null() {
        return;
    }
    let line_count = nvim_syn_buf_get_ml_line_count();
    let rows = nvim_syn_get_rows();

    // Compute desired length
    let len = (line_count / SST_DIST + rows * 2).clamp(SST_MIN_ENTRIES, SST_MAX_ENTRIES);

    let sst_len = nvim_synblock_get_sst_len(block);
    let freecount = nvim_synblock_get_sst_freecount(block);

    if sst_len > len * 2 || sst_len < len {
        // Allocate 50% too much to avoid frequent reallocation.
        let line_count2 = line_count;
        let new_len_raw = (line_count2 + line_count2 / 2) / SST_DIST + rows * 2;
        let mut new_len = new_len_raw.clamp(SST_MIN_ENTRIES, SST_MAX_ENTRIES);

        if nvim_synblock_has_sst_array(block) != 0 {
            // When shrinking, cleanup until all valid entries fit.
            let used_plus_margin = sst_len - freecount + 2;
            while used_plus_margin > new_len && rs_syn_stack_cleanup() != 0 {}
            // Ensure minimum size to hold existing entries.
            let min_needed =
                nvim_synblock_get_sst_len(block) - nvim_synblock_get_sst_freecount(block) + 2;
            if new_len < min_needed {
                new_len = min_needed;
            }
        }

        // Delegate actual array allocation/copying to the Rust implementation.
        crate::state_entry::rs_syn_do_stack_realloc(new_len);
    }
}

/// Adjust cached entries in one synblock after buffer changes.
///
/// Replaces static C `syn_stack_apply_changes_block`.
///
/// # Safety
/// Accesses C global state; must be called from main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_stack_apply_changes_block(block: SynBlockHandle, buf: BufHandle) {
    if block.is_null() || buf.is_null() {
        return;
    }
    let linebreaks = nvim_synblock_get_sync_linebreaks(block);
    let mod_top = nvim_buf_get_mod_top(buf);
    let mod_bot = nvim_buf_get_mod_bot(buf);
    let mod_xlines = nvim_buf_get_mod_xlines(buf);

    let mut prev = SynStateHandle::null();
    let mut p = nvim_synblock_get_sst_first(block);

    while !p.is_null() {
        let p_lnum = nvim_synstate_get_lnum(p);
        let next = nvim_synstate_get_next(p);

        if p_lnum + linebreaks > mod_top {
            let n = p_lnum + mod_xlines;
            if n <= mod_bot {
                // This state is inside the changed area - remove it.
                if prev.is_null() {
                    nvim_synblock_set_sst_first(block, next);
                } else {
                    nvim_synstate_set_next(prev, next);
                }
                rs_syn_stack_free_entry(block, p);
                p = next;
                continue;
            }

            // This state is below the changed area.
            let change_lnum = nvim_synstate_get_change_lnum(p);
            let new_change_lnum = if change_lnum != 0 && change_lnum > mod_top {
                if change_lnum + mod_xlines > mod_top {
                    change_lnum + mod_xlines
                } else {
                    mod_top
                }
            } else {
                change_lnum
            };

            let final_change_lnum = if new_change_lnum == 0 || new_change_lnum < mod_bot {
                mod_bot
            } else {
                new_change_lnum
            };
            nvim_synstate_set_change_lnum(p, final_change_lnum);
            nvim_synstate_set_lnum(p, n);
        }

        prev = p;
        p = next;
    }
}

/// Check for changes in syn_buf to affect stored syntax states.
/// Called from update_screen() before screen update, once per displayed buffer.
///
/// Replaces public C `syn_stack_apply_changes`.
///
/// # Safety
/// Accesses C global state; must be called from main thread.
#[export_name = "syn_stack_apply_changes"]
pub unsafe extern "C" fn rs_syn_stack_apply_changes(buf: BufHandle) {
    if buf.is_null() {
        return;
    }
    let b_s = nvim_buf_get_b_s(buf);
    rs_syn_stack_apply_changes_block(b_s, buf);
    // FOR_ALL_WINDOWS_IN_TAB stays in C
    nvim_syn_apply_changes_for_windows(buf);
}

// =============================================================================
// Cache lookup and storage
// =============================================================================

/// Find a synstate entry for the given line number.
/// Returns the entry at or before the line, or null if none found.
#[must_use]
pub fn stack_find_entry(lnum: i32) -> SynStateHandle {
    unsafe { rs_syn_stack_find_entry(lnum) }
}

/// Find a synstate entry for the given line number (alternate interface).
#[must_use]
pub fn stack_find_entry_ptr(lnum: i32) -> SynStateHandle {
    unsafe { rs_syn_stack_find_entry(lnum) }
}

/// Remove a synstate entry from the cache.
pub fn stack_remove_entry(sp: SynStateHandle) {
    if !sp.is_null() {
        unsafe { crate::state_entry::rs_syn_stack_remove_entry(sp) }
    }
}

/// Allocate a new synstate entry at the given line number.
/// The entry is inserted after the given state (or at the beginning if null).
#[must_use]
pub fn stack_alloc_entry(lnum: i32, after: SynStateHandle) -> SynStateHandle {
    unsafe { crate::state_entry::rs_syn_stack_alloc_entry(lnum, after) }
}

/// Store the current state to a synstate entry.
pub fn store_state_to_entry(sp: SynStateHandle) {
    if !sp.is_null() {
        unsafe { crate::state_entry::rs_syn_store_state_to_entry(sp) }
    }
}

/// Allocate the syntax stack array (b_sst_array).
pub fn stack_alloc() {
    unsafe { rs_syn_stack_alloc() }
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
    let lnum = crate::statics::CURRENT_LNUM;
    let state_len = crate::statics::CURRENT_STATE.ga_len;

    // Find existing entry at or before current line
    let sp = rs_syn_stack_find_entry(lnum);

    // Check if current state contains items that span across lines
    // If so, we can't use this state - it's not valid for line boundaries
    let mut has_spanning_item = false;
    for i in (0..state_len).rev() {
        if crate::state_ops::rs_syn_state_item_spans_line(i, lnum) != 0 {
            has_spanning_item = true;
            break;
        }
    }

    if has_spanning_item {
        // Current state spans lines, can't store it
        // If there was an existing entry at this line, remove it
        if !sp.is_null() {
            crate::state_entry::rs_syn_stack_remove_entry(sp);
        }
        crate::statics::CURRENT_STATE_STORED = 1;
        return SynStateHandle::null();
    }

    // Determine if we need to allocate a new entry
    let entry = if sp.is_null() || nvim_synstate_get_lnum(sp) != lnum {
        // Need to allocate a new entry
        crate::state_entry::rs_syn_stack_alloc_entry(lnum, sp)
    } else {
        // Reuse existing entry
        sp
    };

    if !entry.is_null() {
        // Store current state to the entry
        crate::state_entry::rs_syn_store_state_to_entry(entry);
    }

    crate::statics::CURRENT_STATE_STORED = 1;
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
    crate::state_ops::rs_syn_clear_current_state();
    nvim_syn_validate_current_state();
    crate::statics::KEEPEND_LEVEL = -1;

    let stacksize = nvim_synstate_get_stacksize(from);
    if stacksize > 0 {
        // Grow current state array
        crate::statics::current_state_grow(stacksize);
        crate::statics::CURRENT_STATE.ga_len = stacksize;

        // Copy each state item
        let mut keepend_level = -1;
        for i in 0..stacksize {
            let bs = nvim_synstate_get_bufstate(from, i);
            if bs.is_null() {
                continue;
            }

            let bs_ptr: *mut BufState = bs.as_ptr();
            let bs_idx = unsafe { (*bs_ptr).bs_idx };
            let bs_flags = unsafe { (*bs_ptr).bs_flags };
            let bs_seqnr = unsafe { (*bs_ptr).bs_seqnr };
            let bs_cchar = unsafe { (*bs_ptr).bs_cchar };
            let extmatch = ExtMatchHandle(unsafe { (*bs_ptr).bs_extmatch as *mut _ });

            // Set the state item (this also sets si_next_list based on pattern)
            crate::state_ops::rs_syn_set_cur_state_item(
                i, bs_idx, bs_flags, bs_seqnr, bs_cchar, extmatch,
            );

            // Track keepend level
            if keepend_level < 0 && (bs_flags & HL_KEEPEND) != 0 {
                keepend_level = i;
            }

            // Update attributes for this item
            nvim_syn_update_si_attr(i);
        }

        crate::statics::KEEPEND_LEVEL = keepend_level;
    }

    // Copy next_list and next_flags from saved state
    let next_list = nvim_synstate_get_next_list(from);
    crate::statics::CURRENT_NEXT_LIST = next_list.0;
    crate::statics::CURRENT_NEXT_FLAGS = nvim_synstate_get_next_flags(from);
    crate::statics::CURRENT_LNUM = nvim_synstate_get_lnum(from);
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
    let current_len = crate::statics::CURRENT_STATE.ga_len;

    // Quick check: stack sizes must match
    if sp_stacksize != current_len {
        return false;
    }

    // Quick check: next_list pointers must match
    let sp_next_list = nvim_synstate_get_next_list(sp);
    let cur_next_list = IdListHandle(crate::statics::CURRENT_NEXT_LIST);
    if sp_next_list.0 != cur_next_list.0 {
        return false;
    }

    // Compare each state item
    let nsubexp = crate::types::NSUBEXP;
    for i in (0..current_len).rev() {
        let bs = nvim_synstate_get_bufstate(sp, i);
        if bs.is_null() {
            return false;
        }

        let cur_si = crate::statics::current_state_item(i);
        if cur_si.is_null() {
            return false;
        }

        // Compare basic fields
        let bs_ptr: *mut BufState = bs.as_ptr();
        let si_ptr: *mut StateItem = cur_si.as_ptr();
        if unsafe { (*bs_ptr).bs_idx != (*si_ptr).si_idx } {
            return false;
        }
        if unsafe { (*bs_ptr).bs_flags != (*si_ptr).si_flags } {
            return false;
        }
        if unsafe { (*bs_ptr).bs_seqnr != (*si_ptr).si_seqnr } {
            return false;
        }
        if unsafe { (*bs_ptr).bs_cchar != (*si_ptr).si_cchar } {
            return false;
        }

        // Compare external matches
        let bs_extmatch = ExtMatchHandle(unsafe { (*bs_ptr).bs_extmatch as *mut _ });
        let si_extmatch = ExtMatchHandle(unsafe { (*si_ptr).si_extmatch as *mut _ });

        // If both are null, they're equal
        if bs_extmatch.is_null() && si_extmatch.is_null() {
            continue;
        }

        // If one is null and the other isn't, check for empty strings
        if bs_extmatch.is_null() || si_extmatch.is_null() {
            // Check if the non-null one has any actual content
            if crate::state_ops::rs_syn_extmatch_strings_equal(bs_extmatch, si_extmatch, 0, nsubexp)
                == 0
            {
                return false;
            }
            continue;
        }

        // Both non-null: compare using Rust comparison function
        if crate::state_ops::rs_syn_extmatch_equal(bs_extmatch, si_extmatch) == 0 {
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
