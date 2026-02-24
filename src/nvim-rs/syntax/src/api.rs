//! Public API for syntax highlighting queries.
//!
//! This module provides the high-level API for syntax highlighting
//! that can be called from C code via cbindgen exports.

use std::ffi::c_int;

use crate::highlight::{syn_id2attr, HlFlags, ResolvedHighlight};
use crate::match_engine::{current_attr, current_id, current_match_result, MatchResult};
use crate::sync::{sync_flags, sync_maxlines, sync_minlines, SyncMethod, SyncSettings};

// =============================================================================
// Public query functions
// =============================================================================

/// Get the highlight attribute for a syntax ID.
///
/// This converts a syntax ID (group number) to the actual
/// highlight attribute used for display.
#[must_use]
pub fn get_syntax_attr(syn_id: i32) -> i32 {
    syn_id2attr(syn_id)
}

/// Get the current match result from the syntax engine.
#[must_use]
pub fn get_current_match() -> MatchResult {
    current_match_result()
}

/// Get the current syntax ID being processed.
#[must_use]
pub fn get_current_syntax_id() -> i32 {
    current_id()
}

/// Get the current attribute being processed.
#[must_use]
pub fn get_current_syntax_attr() -> i32 {
    current_attr()
}

// =============================================================================
// Sync query functions
// =============================================================================

/// Get the sync method for the current syntax block.
#[must_use]
pub fn get_sync_method() -> SyncMethod {
    SyncMethod::from_flags(sync_flags())
}

/// Get sync settings for the current syntax block.
#[must_use]
pub fn get_sync_settings() -> SyncSettings {
    SyncSettings::current()
}

/// Get the sync minlines setting.
#[must_use]
pub fn get_sync_minlines() -> i32 {
    sync_minlines()
}

/// Get the sync maxlines setting.
#[must_use]
pub fn get_sync_maxlines() -> i32 {
    sync_maxlines()
}

// =============================================================================
// Highlight query functions
// =============================================================================

/// Resolve a syntax ID to full highlight information.
#[must_use]
pub fn resolve_syntax_highlight(syn_id: i32) -> ResolvedHighlight {
    ResolvedHighlight::from_syn_id(syn_id)
}

/// Extract HL flags from a raw flags value.
#[must_use]
pub fn extract_hl_flags(flags: i32) -> HlFlags {
    HlFlags::from_raw(flags)
}

// =============================================================================
// Syntax info struct for API consumers
// =============================================================================

/// Complete syntax information from current state.
#[derive(Debug, Clone, Copy, Default)]
pub struct SyntaxInfo {
    /// Syntax ID (group number).
    pub syn_id: i32,
    /// Transparent syntax ID.
    pub trans_id: i32,
    /// Highlight attribute.
    pub attr: i32,
    /// HL flags.
    pub flags: i32,
    /// Conceal character (0 if none).
    pub cchar: i32,
}

impl SyntaxInfo {
    /// Create syntax info from current engine state.
    #[must_use]
    pub fn from_current_state() -> Self {
        let syn_id = current_id();
        let attr = current_attr();

        Self {
            syn_id,
            trans_id: syn_id, // Transparent ID matches when getting from current state
            attr,
            flags: 0,
            cchar: 0,
        }
    }

    /// Check if there is syntax highlighting at this position.
    #[must_use]
    pub const fn has_syntax(&self) -> bool {
        self.syn_id != 0
    }

    /// Check if there is a highlight attribute.
    #[must_use]
    pub const fn has_attr(&self) -> bool {
        self.attr != 0
    }
}

// =============================================================================
// Phase 3: syn_get_id and query API (Rust implementations)
// =============================================================================

use crate::types::{StateItemHandle, WinHandle, BufHandle, HL_CONCEAL};

extern "C" {
    fn nvim_syn_get_win() -> WinHandle;
    fn nvim_syn_get_buf() -> BufHandle;
    fn nvim_syn_win_get_buffer_ptr(wp: WinHandle) -> BufHandle;
    fn nvim_syn_get_current_lnum() -> c_int;
    fn nvim_syn_get_current_col() -> c_int;
    fn nvim_syn_get_current_id() -> c_int;
    fn nvim_syn_get_current_trans_id() -> c_int;
    fn nvim_syn_get_current_flags() -> c_int;
    fn nvim_syn_get_current_seqnr() -> c_int;
    fn nvim_syn_set_next_match_idx(idx: c_int);
    fn nvim_syn_get_current_state_len() -> c_int;
    fn nvim_syn_invalidate_current_state();
    fn nvim_syn_set_current_col(col: c_int);
    fn nvim_syn_get_cur_state(idx: c_int) -> StateItemHandle;
    fn nvim_stateitem_get_id(item: StateItemHandle) -> c_int;
}

/// MAXCOL: large column value used to invalidate column state.
const MAXCOL: i32 = 0x7fff_ffff;

/// Core implementation of `syn_get_id` -- now in Rust.
///
/// Returns the syntax ID at position (lnum, col) in window `wp`.
/// If `trans != 0`, returns the transparent ID.
/// Writes spell-checking flag to `spellp` if non-null.
///
/// # Safety
/// Must be called from the main thread.
pub unsafe fn syn_get_id_impl(
    wp: WinHandle,
    lnum: c_int,
    col: c_int,
    trans: c_int,
    spellp: *mut c_int,
    keep_state: c_int,
) -> c_int {
    let syn_win = nvim_syn_get_win();
    let syn_buf = nvim_syn_get_buf();
    let wp_buf = nvim_syn_win_get_buffer_ptr(wp);
    let current_lnum = nvim_syn_get_current_lnum();
    let current_col = nvim_syn_get_current_col();

    if wp.0 != syn_win.0 || wp_buf.0 != syn_buf.0 || lnum != current_lnum || col < current_col {
        crate::buffer::start_syntax(wp, lnum);
    } else if col > current_col {
        nvim_syn_set_next_match_idx(-1);
    }

    let result = crate::highlight::get_syntax_attr(col, keep_state != 0);
    if !spellp.is_null() {
        *spellp = if result.can_spell { 1 } else { 0 };
    }

    if trans != 0 {
        nvim_syn_get_current_trans_id()
    } else {
        nvim_syn_get_current_id()
    }
}

/// Core implementation of `get_syntax_info`.
///
/// Stores the current sequence number in `seqnrp` and returns the current flags.
///
/// # Safety
/// Must be called right after `syn_get_id_impl` (or equivalent).
pub unsafe fn get_syntax_info_impl(seqnrp: *mut c_int) -> c_int {
    *seqnrp = nvim_syn_get_current_seqnr();
    nvim_syn_get_current_flags()
}

/// Core implementation of `syn_get_concealed_id`.
///
/// # Safety
/// Must be called from the main thread.
pub unsafe fn syn_get_concealed_id_impl(wp: WinHandle, lnum: c_int, col: c_int) -> c_int {
    syn_get_id_impl(wp, lnum, col, 0, std::ptr::null_mut(), 0);
    let mut seqnr: c_int = 0;
    let syntax_flags = get_syntax_info_impl(&mut seqnr);
    if (syntax_flags as u32) & (HL_CONCEAL as u32) != 0 {
        seqnr
    } else {
        0
    }
}

/// Core implementation of `syn_get_stack_item`.
///
/// Returns the syntax ID at state stack position `i`, or -1 if out of range.
///
/// # Safety
/// Must be called after `syn_get_id_impl` with `keep_state = 1`.
pub unsafe fn syn_get_stack_item_impl(i: c_int) -> c_int {
    if i >= nvim_syn_get_current_state_len() {
        nvim_syn_invalidate_current_state();
        nvim_syn_set_current_col(MAXCOL);
        return -1;
    }
    let item = nvim_syn_get_cur_state(i);
    nvim_stateitem_get_id(item)
}

// =============================================================================
// C API exports
// =============================================================================

/// Get the highlight attribute for a syntax ID (exported for cbindgen).
#[no_mangle]
pub extern "C" fn rs_syntax_id_to_attr(syn_id: c_int) -> c_int {
    get_syntax_attr(syn_id)
}

/// Get syntax ID at a file position -- Rust implementation of `syn_get_id`.
///
/// # Safety
/// Must be called from the main thread during syntax highlighting.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_get_id(
    wp: WinHandle,
    lnum: c_int,
    col: c_int,
    trans: c_int,
    spellp: *mut c_int,
    keep_state: c_int,
) -> c_int {
    syn_get_id_impl(wp, lnum, col, trans, spellp, keep_state)
}

/// Get extra syntax info -- Rust implementation of `get_syntax_info`.
///
/// # Safety
/// Must be called right after `rs_syn_get_id` or equivalent.
#[no_mangle]
pub unsafe extern "C" fn rs_get_syntax_info(seqnrp: *mut c_int) -> c_int {
    if seqnrp.is_null() {
        return 0;
    }
    get_syntax_info_impl(seqnrp)
}

/// Get concealed ID -- Rust implementation of `syn_get_concealed_id`.
///
/// # Safety
/// Must be called from the main thread.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_get_concealed_id(wp: WinHandle, lnum: c_int, col: c_int) -> c_int {
    syn_get_concealed_id_impl(wp, lnum, col)
}

/// Get stack item ID -- Rust implementation of `syn_get_stack_item`.
///
/// # Safety
/// Must be called after `rs_syn_get_id` with `keep_state = 1`.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_get_stack_item(i: c_int) -> c_int {
    syn_get_stack_item_impl(i)
}

/// Get the current syntax ID being processed (exported for cbindgen).
#[no_mangle]
pub extern "C" fn rs_syntax_current_id() -> c_int {
    get_current_syntax_id()
}

/// Get the current attribute being processed (exported for cbindgen).
#[no_mangle]
pub extern "C" fn rs_syntax_current_attr() -> c_int {
    get_current_syntax_attr()
}

/// Get the sync minlines setting (exported for cbindgen).
#[no_mangle]
pub extern "C" fn rs_syntax_sync_minlines() -> c_int {
    get_sync_minlines()
}

/// Get the sync maxlines setting (exported for cbindgen).
#[no_mangle]
pub extern "C" fn rs_syntax_sync_maxlines() -> c_int {
    get_sync_maxlines()
}

/// Check if the sync method uses C comments (exported for cbindgen).
#[no_mangle]
pub extern "C" fn rs_syntax_sync_is_ccomment() -> c_int {
    if matches!(get_sync_method(), SyncMethod::CComment) {
        1
    } else {
        0
    }
}

/// Check if the sync method uses match patterns (exported for cbindgen).
#[no_mangle]
pub extern "C" fn rs_syntax_sync_is_match() -> c_int {
    if matches!(get_sync_method(), SyncMethod::Match) {
        1
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syntax_info_default() {
        let info = SyntaxInfo::default();
        assert_eq!(info.syn_id, 0);
        assert_eq!(info.trans_id, 0);
        assert_eq!(info.attr, 0);
        assert_eq!(info.flags, 0);
        assert_eq!(info.cchar, 0);
        assert!(!info.has_syntax());
        assert!(!info.has_attr());
    }

    #[test]
    fn test_syntax_info_with_values() {
        let info = SyntaxInfo {
            syn_id: 5,
            trans_id: 5,
            attr: 10,
            flags: 0,
            cchar: 0,
        };
        assert!(info.has_syntax());
        assert!(info.has_attr());
    }

    #[test]
    fn test_hl_flags_extraction() {
        let flags = extract_hl_flags(0);
        assert!(!flags.oneline);
        assert!(!flags.keepend);
        assert!(!flags.is_match);
        assert!(!flags.conceal);
    }
}
