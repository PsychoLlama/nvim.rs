//! Migrated C static variables for the syntax engine.
//!
//! These `static mut` variables correspond to file-scope statics that were
//! previously in `syntax_accessors.c`. Moving them to Rust eliminates the
//! need for C accessor functions to bridge the FFI boundary.
//!
//! # Safety
//!
//! All access to these statics must be done in `unsafe` blocks. The syntax
//! engine is single-threaded (runs on the main Neovim thread), so no
//! synchronization is needed.

use std::ffi::{c_int, c_void};

use crate::ffi_types::{LPos, SynCluster, SynPat};
use crate::types::{StateItemHandle, SynBlockHandle};

// =============================================================================
// Phase 1: Scalar statics migrated from syntax_accessors.c
// =============================================================================

/// Attribute of current syntax word (mapped from C `current_attr`).
#[no_mangle]
pub static mut CURRENT_ATTR: c_int = 0;

/// ID of current char for syn_get_id() (mapped from C `current_id`).
#[no_mangle]
pub static mut CURRENT_ID: c_int = 0;

/// Transparency-removed ID (mapped from C `current_trans_id`).
#[no_mangle]
pub static mut CURRENT_TRANS_ID: c_int = 0;

/// Current highlight flags (mapped from C `current_flags`).
#[no_mangle]
pub static mut CURRENT_FLAGS: c_int = 0;

/// Current sequence number (mapped from C `current_seqnr`).
#[no_mangle]
pub static mut CURRENT_SEQNR: c_int = 0;

/// Current substitution character (mapped from C `current_sub_char`).
#[no_mangle]
pub static mut CURRENT_SUB_CHAR: c_int = 0;

/// Line number of current state (mapped from C `current_lnum`).
#[no_mangle]
pub static mut CURRENT_LNUM: c_int = 0;

/// Column of current state (mapped from C `current_col`).
#[no_mangle]
pub static mut CURRENT_COL: c_int = 0;

/// True if current line has been finished (mapped from C `current_finished`).
/// Stored as int (0/1) to allow atomic access from both Rust and C during migration.
#[no_mangle]
pub static mut CURRENT_FINISHED: c_int = 0;

/// True if stored current state after setting current_finished
/// (mapped from C `current_state_stored`).
#[no_mangle]
pub static mut CURRENT_STATE_STORED: c_int = 0;

/// Flags for current_next_list (mapped from C `current_next_flags`).
#[no_mangle]
pub static mut CURRENT_NEXT_FLAGS: c_int = 0;

/// Unique number for current line (mapped from C `current_line_id`).
#[no_mangle]
pub static mut CURRENT_LINE_ID: c_int = 0;

/// Unique tag for `:syn include`'d rules (mapped from C `current_syn_inc_tag`).
#[no_mangle]
pub static mut CURRENT_SYN_INC_TAG: c_int = 0;

/// Running tag counter for `:syn include` (mapped from C `running_syn_inc_tag`).
#[no_mangle]
pub static mut RUNNING_SYN_INC_TAG: c_int = 0;

/// Level of first keepend item on state stack, -1 if none
/// (mapped from C `keepend_level`).
#[no_mangle]
pub static mut KEEPEND_LEVEL: c_int = -1;

/// Value to use for si_seqnr (mapped from C `next_seqnr`).
#[no_mangle]
pub static mut NEXT_SEQNR: c_int = 1;

/// True if syntax timing is enabled (mapped from C `syn_time_on`).
#[no_mangle]
pub static mut SYN_TIME_ON: c_int = 0;

/// True if `:syntax on/off` was called (mapped from C `did_syntax_onoff`).
#[no_mangle]
pub static mut DID_SYNTAX_ONOFF: c_int = 0;

/// What to expand for `:syn` completion (mapped from C `expand_what`).
#[no_mangle]
pub static mut EXPAND_WHAT: c_int = 0;

// =============================================================================
// Phase 2: Pointer and struct statics migrated from syntax_accessors.c
// =============================================================================

/// Column for start of next match (mapped from C `next_match_col`).
/// Value MAXCOL (0x7fffffff) means no match found.
#[no_mangle]
pub static mut NEXT_MATCH_COL: c_int = 0;

/// Index of matched item (mapped from C `next_match_idx`).
/// Value -1 means not tried yet.
#[no_mangle]
pub static mut NEXT_MATCH_IDX: c_int = -1;

/// Flags for next match (mapped from C `next_match_flags`).
#[no_mangle]
pub static mut NEXT_MATCH_FLAGS: c_int = 0;

/// ID of group for end pattern or zero (mapped from C `next_match_end_idx`).
#[no_mangle]
pub static mut NEXT_MATCH_END_IDX: c_int = 0;

/// Position for end of next match (mapped from C `next_match_m_endpos`).
#[no_mangle]
pub static mut NEXT_MATCH_M_ENDPOS: LPos = LPos { lnum: 0, col: 0 };

/// Position for highlight start of next match (mapped from C `next_match_h_startpos`).
#[no_mangle]
pub static mut NEXT_MATCH_H_STARTPOS: LPos = LPos { lnum: 0, col: 0 };

/// Position for highlight end of next match (mapped from C `next_match_h_endpos`).
#[no_mangle]
pub static mut NEXT_MATCH_H_ENDPOS: LPos = LPos { lnum: 0, col: 0 };

/// End of start pattern position (mapped from C `next_match_eos_pos`).
#[no_mangle]
pub static mut NEXT_MATCH_EOS_POS: LPos = LPos { lnum: 0, col: 0 };

/// Position for end of end pattern (mapped from C `next_match_eoe_pos`).
#[no_mangle]
pub static mut NEXT_MATCH_EOE_POS: LPos = LPos { lnum: 0, col: 0 };

/// Current nextgroup list pointer (mapped from C `current_next_list`).
/// NULL when not active.
#[no_mangle]
pub static mut CURRENT_NEXT_LIST: *mut i16 = std::ptr::null_mut();

/// Extmatch for next match (mapped from C `next_match_extmatch`).
/// Opaque pointer -- NULL when no match.
#[no_mangle]
pub static mut NEXT_MATCH_EXTMATCH: *mut std::ffi::c_void = std::ptr::null_mut();

// =============================================================================
// Phase 3: current_state garray migrated from syntax_accessors.c
// =============================================================================

/// C-compatible growing array (mirrors `garray_T` layout exactly).
///
/// Layout matches `garray_T` from `garray_defs.h`:
/// ```c
/// typedef struct {
///   int ga_len;       // current number of items used
///   int ga_maxlen;    // maximum number of items possible
///   int ga_itemsize;  // sizeof(item)
///   int ga_growsize;  // number of items to grow each time
///   void *ga_data;    // pointer to the first item
/// } garray_T;
/// ```
#[repr(C)]
pub struct GArray {
    pub ga_len: c_int,
    pub ga_maxlen: c_int,
    pub ga_itemsize: c_int,
    pub ga_growsize: c_int,
    pub ga_data: *mut c_void,
}

/// The syntax state stack (mapped from C `current_state`).
/// Initial value matches `GA_EMPTY_INIT_VALUE = { 0, 0, 0, 1, NULL }`.
#[no_mangle]
pub static mut CURRENT_STATE: GArray = GArray {
    ga_len: 0,
    ga_maxlen: 0,
    ga_itemsize: 0,
    ga_growsize: 1,
    ga_data: std::ptr::null_mut(),
};

extern "C" {
    /// Grow a garray by at least `n` items.
    fn ga_grow(gap: *mut GArray, n: c_int);

    /// Set the growsize of a garray.
    fn ga_set_growsize(gap: *mut GArray, growsize: c_int);

    /// Get base pointer to synpat_T array for a synblock.
    /// Returns NULL if block is NULL or patterns array is empty.
    fn nvim_synblock_get_patterns_ga_data(block: SynBlockHandle) -> *mut SynPat;

    /// Get base pointer to syn_cluster_T array for a synblock.
    /// Returns NULL if block is NULL or clusters array is empty.
    fn nvim_synblock_get_clusters_ga_data(block: SynBlockHandle) -> *mut SynCluster;
}

// =============================================================================
// Phase 3: current_state helper functions
// =============================================================================

/// Return a pointer to CURRENT_STATE item at `index`, or null if out of bounds.
///
/// # Safety
/// Must be called from the main thread.
#[inline]
pub unsafe fn current_state_item(index: c_int) -> StateItemHandle {
    if index < 0 || index >= CURRENT_STATE.ga_len {
        return StateItemHandle::null();
    }
    let base = CURRENT_STATE.ga_data as *mut crate::ffi_types::StateItem;
    StateItemHandle::from_ptr(base.add(index as usize))
}

/// Return a pointer to the top item on the state stack, or null if empty.
///
/// # Safety
/// Must be called from the main thread.
#[inline]
pub unsafe fn current_state_top() -> StateItemHandle {
    if CURRENT_STATE.ga_len == 0 {
        return StateItemHandle::null();
    }
    current_state_item(CURRENT_STATE.ga_len - 1)
}

/// Return true if the current state stack is valid (ga_itemsize != 0).
#[inline]
pub unsafe fn current_state_is_valid() -> bool {
    CURRENT_STATE.ga_itemsize != 0
}

/// Return true if the current state stack is empty (ga_len == 0).
#[inline]
pub unsafe fn current_state_is_empty() -> bool {
    CURRENT_STATE.ga_len <= 0
}

/// Grow the current state stack by at least `n` items.
///
/// # Safety
/// Must be called from the main thread. Invalidates all stateitem pointers.
#[inline]
pub unsafe fn current_state_grow(n: c_int) {
    ga_grow(&raw mut CURRENT_STATE, n);
}

/// Set ga_itemsize and ga_growsize to validate the current state.
/// Equivalent to C `validate_current_state()`.
///
/// # Safety
/// Must be called from the main thread.
#[inline]
pub unsafe fn current_state_validate() {
    CURRENT_STATE.ga_itemsize = std::mem::size_of::<crate::ffi_types::StateItem>() as c_int;
    ga_set_growsize(&raw mut CURRENT_STATE, 3);
}

/// Append a new zeroed StateItem to CURRENT_STATE and return a handle to it.
/// Equivalent to `GA_APPEND_VIA_PTR(stateitem_T, &current_state)` + `CLEAR_POINTER`.
///
/// # Safety
/// Must be called from the main thread. Invalidates all existing stateitem pointers.
pub unsafe fn current_state_append() -> StateItemHandle {
    ga_grow(&raw mut CURRENT_STATE, 1);
    let idx = CURRENT_STATE.ga_len;
    CURRENT_STATE.ga_len += 1;
    let base = CURRENT_STATE.ga_data as *mut crate::ffi_types::StateItem;
    let p = base.add(idx as usize);
    std::ptr::write_bytes(p, 0, 1);
    StateItemHandle::from_ptr(p)
}

// =============================================================================
// Phase 4: SynPat / SynCluster direct field access helpers
// =============================================================================

/// Get pointer to SynPat at `idx` in `block`'s patterns array.
/// Returns null if block is null, idx is out of range, or array is empty.
///
/// # Safety
/// Must be called from the main thread. Do NOT cache across ga_grow calls.
#[inline]
pub unsafe fn syn_item_at(block: SynBlockHandle, idx: c_int) -> *mut SynPat {
    if block.is_null() || idx < 0 {
        return std::ptr::null_mut();
    }
    let base = nvim_synblock_get_patterns_ga_data(block);
    if base.is_null() {
        return std::ptr::null_mut();
    }
    base.add(idx as usize)
}

/// Get pointer to SynCluster at `idx` in `block`'s clusters array.
/// Returns null if block is null, idx is out of range, or array is empty.
///
/// # Safety
/// Must be called from the main thread. Do NOT cache across ga_grow calls.
#[inline]
pub unsafe fn syn_cluster_at(block: SynBlockHandle, idx: c_int) -> *mut SynCluster {
    if block.is_null() || idx < 0 {
        return std::ptr::null_mut();
    }
    let base = nvim_synblock_get_clusters_ga_data(block);
    if base.is_null() {
        return std::ptr::null_mut();
    }
    base.add(idx as usize)
}
