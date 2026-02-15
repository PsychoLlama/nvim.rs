//! Highlight integration for syntax highlighting.
//!
//! This module handles:
//! - Connect syntax IDs to highlight groups
//! - Attribute resolution
//! - Conceal handling

use std::ffi::c_int;

use crate::types::{
    KeyEntryHandle, StateItemHandle, SynBlockHandle, SynPatHandle, WinHandle, HL_CONCEAL,
    HL_CONCEALENDS, HL_EXTEND, HL_KEEPEND, HL_MATCH, HL_MATCHCONT, HL_ONELINE,
};

// =============================================================================
// FFI declarations for highlight operations
// =============================================================================

extern "C" {
    // Syntax ID to attribute conversion
    fn nvim_syn_id2attr_wrapper(syn_id: c_int) -> c_int;

    // Synblock conceal settings
    fn nvim_synblock_get_conceal(block: SynBlockHandle) -> c_int;
    fn nvim_synblock_get_conceal_setting(block: SynBlockHandle) -> c_int;

    // Pattern highlight group
    fn nvim_synpat_get_hl_group(pat: SynPatHandle) -> c_int;
    fn nvim_synpat_get_cchar(pat: SynPatHandle) -> c_int;

    // State item conceal char
    fn nvim_stateitem_get_cchar(item: StateItemHandle) -> c_int;

    // Keyword conceal char
    fn nvim_keyentry_get_char(ke: KeyEntryHandle) -> c_int;

    // Current HL flags
    fn nvim_syn_get_hl_oneline() -> c_int;
    fn nvim_syn_get_hl_keepend() -> c_int;
    fn nvim_syn_get_hl_match() -> c_int;
    fn nvim_syn_get_hl_conceal() -> c_int;
    fn nvim_syn_get_hl_concealends() -> c_int;
    fn nvim_syn_get_hl_matchcont() -> c_int;
    fn nvim_syn_get_hl_extend() -> c_int;

    // Concealed position check
    fn syn_get_concealed_id(wp: WinHandle, lnum: c_int, col: c_int) -> c_int;

    // Phase 32.4: Line highlighting
    fn nvim_get_syntax_attr(col: c_int, can_spell: *mut c_int, keep_state: c_int) -> c_int;
    fn nvim_syn_get_current_col() -> c_int;
    fn nvim_syn_set_current_col(col: c_int);
    fn nvim_syn_get_current_finished() -> c_int;
    fn nvim_syn_get_current_state_stored() -> c_int;
    fn nvim_synblock_get_syn_spell(block: SynBlockHandle) -> c_int;
    fn nvim_buf_get_synmaxcol(buf: crate::types::BufHandle) -> c_int;
    fn nvim_syn_current_state_valid() -> c_int;
    fn nvim_syn_ensure_current_state_valid();
    fn nvim_syn_get_current_line() -> *const std::ffi::c_char;
    fn nvim_syn_get_next_match_attr() -> c_int;
    fn nvim_syn_get_next_match_idx() -> c_int;
    fn nvim_syn_get_next_match_col() -> c_int;

    // get_syntax_attr dependencies
    fn nvim_synblock_get_spell_cluster_id(block: SynBlockHandle) -> c_int;
    fn nvim_synblock_has_sst_array(block: SynBlockHandle) -> c_int;
    fn nvim_syn_get_syn_block() -> SynBlockHandle;
    fn nvim_syn_get_buf() -> crate::types::BufHandle;
    fn nvim_syn_clear_current_state();
    fn nvim_syn_set_current_id(id: c_int);
    fn nvim_syn_set_current_trans_id(id: c_int);
    fn nvim_syn_set_current_flags(flags: c_int);
    fn nvim_syn_set_current_seqnr(seqnr: c_int);

    fn rs_syn_current_attr_impl(
        syncing: c_int,
        displaying: c_int,
        can_spell: *mut c_int,
        keep_state: c_int,
    ) -> c_int;
}

// =============================================================================
// Syntax ID to attribute conversion
// =============================================================================

/// Convert a syntax ID to a highlight attribute.
///
/// This is the main function for resolving syntax highlighting
/// to actual display attributes.
#[must_use]
pub fn syn_id2attr(syn_id: i32) -> i32 {
    unsafe { nvim_syn_id2attr_wrapper(syn_id) }
}

// =============================================================================
// Synblock conceal settings
// =============================================================================

/// Get the conceal setting for a synblock.
#[must_use]
pub fn synblock_conceal(block: SynBlockHandle) -> bool {
    if block.is_null() {
        return false;
    }
    unsafe { nvim_synblock_get_conceal(block) != 0 }
}

/// Get the conceal setting for a synblock (same as above).
#[must_use]
pub fn synblock_conceal_setting(block: SynBlockHandle) -> bool {
    if block.is_null() {
        return false;
    }
    unsafe { nvim_synblock_get_conceal_setting(block) != 0 }
}

// =============================================================================
// Pattern highlight accessors
// =============================================================================

/// Get the highlight group for a pattern.
#[must_use]
pub fn synpat_hl_group(pat: SynPatHandle) -> i32 {
    if pat.is_null() {
        return 0;
    }
    unsafe { nvim_synpat_get_hl_group(pat) }
}

/// Get the conceal character for a pattern.
#[must_use]
pub fn synpat_cchar(pat: SynPatHandle) -> i32 {
    if pat.is_null() {
        return 0;
    }
    unsafe { nvim_synpat_get_cchar(pat) }
}

// =============================================================================
// State item conceal
// =============================================================================

/// Get the conceal character for a state item.
#[must_use]
pub fn stateitem_cchar(item: StateItemHandle) -> i32 {
    if item.is_null() {
        return 0;
    }
    unsafe { nvim_stateitem_get_cchar(item) }
}

// =============================================================================
// Keyword conceal
// =============================================================================

/// Get the conceal character for a keyword entry.
#[must_use]
pub fn keyentry_cchar(ke: KeyEntryHandle) -> i32 {
    if ke.is_null() {
        return 0;
    }
    unsafe { nvim_keyentry_get_char(ke) }
}

// =============================================================================
// Current HL flags
// =============================================================================

/// Check if HL_ONELINE is currently set.
#[must_use]
pub fn hl_oneline() -> bool {
    unsafe { nvim_syn_get_hl_oneline() != 0 }
}

/// Check if HL_KEEPEND is currently set.
#[must_use]
pub fn hl_keepend() -> bool {
    unsafe { nvim_syn_get_hl_keepend() != 0 }
}

/// Check if HL_MATCH is currently set.
#[must_use]
pub fn hl_match() -> bool {
    unsafe { nvim_syn_get_hl_match() != 0 }
}

/// Check if HL_CONCEAL is currently set.
#[must_use]
pub fn hl_conceal() -> bool {
    unsafe { nvim_syn_get_hl_conceal() != 0 }
}

/// Check if HL_CONCEALENDS is currently set.
#[must_use]
pub fn hl_concealends() -> bool {
    unsafe { nvim_syn_get_hl_concealends() != 0 }
}

/// Check if HL_MATCHCONT is currently set.
#[must_use]
pub fn hl_matchcont() -> bool {
    unsafe { nvim_syn_get_hl_matchcont() != 0 }
}

/// Check if HL_EXTEND is currently set.
#[must_use]
pub fn hl_extend() -> bool {
    unsafe { nvim_syn_get_hl_extend() != 0 }
}

// =============================================================================
// Concealed position check
// =============================================================================

/// Get the sequence number if a position is concealed.
///
/// # Arguments
/// * `wp` - Window handle
/// * `lnum` - Line number
/// * `col` - Column number
///
/// # Returns
/// Sequence number if concealed, 0 otherwise.
#[must_use]
pub fn get_concealed_id(wp: WinHandle, lnum: i32, col: i32) -> i32 {
    if wp.is_null() {
        return 0;
    }
    unsafe { syn_get_concealed_id(wp, lnum, col) }
}

/// Check if a position is concealed.
#[must_use]
pub fn is_concealed(wp: WinHandle, lnum: i32, col: i32) -> bool {
    get_concealed_id(wp, lnum, col) != 0
}

// =============================================================================
// HL flag helpers
// =============================================================================

/// Extract HL flags from a flags value.
#[derive(Debug, Clone, Copy, Default)]
pub struct HlFlags {
    /// Item continues on one line only.
    pub oneline: bool,
    /// Keep end position even when contained item ends earlier.
    pub keepend: bool,
    /// This is a match item.
    pub is_match: bool,
    /// Item is concealed.
    pub conceal: bool,
    /// Conceal start and end of region.
    pub concealends: bool,
    /// Item continues on next line (match continuation).
    pub matchcont: bool,
    /// Item extends beyond 'synmaxcol'.
    pub extend: bool,
}

impl HlFlags {
    /// Create HlFlags from a raw flags value.
    #[must_use]
    pub const fn from_raw(flags: i32) -> Self {
        Self {
            oneline: (flags & HL_ONELINE) != 0,
            keepend: (flags & HL_KEEPEND) != 0,
            is_match: (flags & HL_MATCH) != 0,
            conceal: (flags & HL_CONCEAL) != 0,
            concealends: (flags & HL_CONCEALENDS) != 0,
            matchcont: (flags & HL_MATCHCONT) != 0,
            extend: (flags & HL_EXTEND) != 0,
        }
    }

    /// Get the current HL flags from global state.
    #[must_use]
    pub fn current() -> Self {
        Self {
            oneline: hl_oneline(),
            keepend: hl_keepend(),
            is_match: hl_match(),
            conceal: hl_conceal(),
            concealends: hl_concealends(),
            matchcont: hl_matchcont(),
            extend: hl_extend(),
        }
    }

    /// Convert back to raw flags.
    #[must_use]
    pub const fn to_raw(&self) -> i32 {
        let mut flags = 0;
        if self.oneline {
            flags |= HL_ONELINE;
        }
        if self.keepend {
            flags |= HL_KEEPEND;
        }
        if self.is_match {
            flags |= HL_MATCH;
        }
        if self.conceal {
            flags |= HL_CONCEAL;
        }
        if self.concealends {
            flags |= HL_CONCEALENDS;
        }
        if self.matchcont {
            flags |= HL_MATCHCONT;
        }
        if self.extend {
            flags |= HL_EXTEND;
        }
        flags
    }
}

// =============================================================================
// Conceal result
// =============================================================================

/// Result of checking a position for concealment.
#[derive(Debug, Clone, Copy)]
pub struct ConcealResult {
    /// Whether the position is concealed.
    pub is_concealed: bool,
    /// The substitute character to display (0 if none).
    pub cchar: i32,
    /// Sequence number for the concealed range.
    pub seqnr: i32,
}

impl ConcealResult {
    /// Create a result indicating the position is not concealed.
    #[must_use]
    pub const fn not_concealed() -> Self {
        Self {
            is_concealed: false,
            cchar: 0,
            seqnr: 0,
        }
    }

    /// Create a result indicating the position is concealed.
    #[must_use]
    pub const fn concealed(cchar: i32, seqnr: i32) -> Self {
        Self {
            is_concealed: true,
            cchar,
            seqnr,
        }
    }
}

// =============================================================================
// Highlight resolution
// =============================================================================

/// Resolved highlight information for a syntax item.
#[derive(Debug, Clone, Copy, Default)]
pub struct ResolvedHighlight {
    /// The highlight attribute ID.
    pub attr: i32,
    /// The syntax ID.
    pub syn_id: i32,
    /// Conceal character (0 if none).
    pub cchar: i32,
    /// HL flags.
    pub flags: i32,
}

impl ResolvedHighlight {
    /// Create a resolved highlight from a syntax ID.
    #[must_use]
    pub fn from_syn_id(syn_id: i32) -> Self {
        Self {
            attr: syn_id2attr(syn_id),
            syn_id,
            cchar: 0,
            flags: 0,
        }
    }

    /// Create a resolved highlight with full information.
    #[must_use]
    pub fn new(syn_id: i32, cchar: i32, flags: i32) -> Self {
        Self {
            attr: syn_id2attr(syn_id),
            syn_id,
            cchar,
            flags,
        }
    }

    /// Check if this highlight is concealed.
    #[must_use]
    pub const fn is_concealed(&self) -> bool {
        (self.flags & HL_CONCEAL) != 0
    }

    /// Get the conceal character to display.
    #[must_use]
    pub const fn conceal_char(&self) -> Option<i32> {
        if self.is_concealed() && self.cchar != 0 {
            Some(self.cchar)
        } else {
            None
        }
    }
}

// =============================================================================
// Phase 32.4: Line highlighting engine
// =============================================================================

/// Result from getting syntax attributes at a column.
#[derive(Debug, Clone, Copy, Default)]
pub struct SyntaxAttrResult {
    /// The highlight attribute.
    pub attr: i32,
    /// Whether spell checking is allowed at this position.
    pub can_spell: bool,
}

/// SYNSPL_DEFAULT and SYNSPL_TOP constants.
const SYNSPL_DEFAULT: c_int = 0;
const SYNSPL_TOP: c_int = 1;

/// Get syntax attributes at a column in the current line.
///
/// This is the main entry point for getting highlighting during redraw.
/// Real Rust implementation replacing the C `get_syntax_attr`.
///
/// # Safety
/// Must be called after `syntax_start` has been called for the current window/line.
#[must_use]
pub unsafe fn get_syntax_attr(col: i32, keep_state: bool) -> SyntaxAttrResult {
    get_syntax_attr_impl(col, keep_state)
}

/// Real implementation of get_syntax_attr.
///
/// # Safety
/// Requires valid C global state.
unsafe fn get_syntax_attr_impl(col: c_int, keep_state: bool) -> SyntaxAttrResult {
    let block = nvim_syn_get_syn_block();
    let buf = nvim_syn_get_buf();

    // Default spell checking value
    let syn_spell = nvim_synblock_get_syn_spell(block);
    let can_spell = if syn_spell == SYNSPL_DEFAULT {
        nvim_synblock_get_spell_cluster_id(block) == 0
    } else {
        syn_spell == SYNSPL_TOP
    };

    // Check for out of memory situation
    if nvim_synblock_has_sst_array(block) == 0 {
        return SyntaxAttrResult { attr: 0, can_spell };
    }

    // After 'synmaxcol' the attribute is always zero.
    let synmaxcol = nvim_buf_get_synmaxcol(buf);
    if synmaxcol > 0 && col >= synmaxcol {
        nvim_syn_clear_current_state();
        nvim_syn_set_current_id(0);
        nvim_syn_set_current_trans_id(0);
        nvim_syn_set_current_flags(0);
        nvim_syn_set_current_seqnr(0);
        return SyntaxAttrResult { attr: 0, can_spell };
    }

    // Make sure current_state is valid
    if nvim_syn_current_state_valid() == 0 {
        nvim_syn_ensure_current_state_valid();
    }

    // Skip from the current column to "col", get the attributes for "col".
    let mut attr = 0;
    let mut spell_result: c_int = if can_spell { 1 } else { 0 };
    let mut current_col = nvim_syn_get_current_col();
    while current_col <= col {
        let ks = if current_col == col {
            keep_state
        } else {
            false
        };
        attr = rs_syn_current_attr_impl(0, 1, &mut spell_result, if ks { 1 } else { 0 });
        current_col = nvim_syn_get_current_col();
        nvim_syn_set_current_col(current_col + 1);
        current_col = nvim_syn_get_current_col();
    }

    SyntaxAttrResult {
        attr,
        can_spell: spell_result != 0,
    }
}

/// Get syntax attributes without spell information.
///
/// # Safety
/// Must be called after `syntax_start` has been called for the current window/line.
#[must_use]
pub unsafe fn get_syntax_attr_simple(col: i32, keep_state: bool) -> i32 {
    get_syntax_attr(col, keep_state).attr
}

/// Get the current column being processed.
#[must_use]
pub fn current_col() -> i32 {
    unsafe { nvim_syn_get_current_col() }
}

/// Set the current column for processing.
pub fn set_current_col(col: i32) {
    unsafe { nvim_syn_set_current_col(col) }
}

/// Check if the current line processing is finished.
#[must_use]
pub fn current_finished() -> bool {
    unsafe { nvim_syn_get_current_finished() != 0 }
}

/// Check if the current state has been stored.
#[must_use]
pub fn current_state_stored() -> bool {
    unsafe { nvim_syn_get_current_state_stored() != 0 }
}

/// Spell checking mode constants.
pub mod syn_spell {
    /// Default: spell check if no @Spell cluster.
    pub const DEFAULT: i32 = 0;
    /// Spell check top-level text only.
    pub const TOP: i32 = 1;
    /// No spell checking.
    pub const NOTOP: i32 = 2;
}

/// Get the spell setting for a synblock.
#[must_use]
pub fn synblock_syn_spell(block: SynBlockHandle) -> i32 {
    if block.is_null() {
        return syn_spell::DEFAULT;
    }
    unsafe { nvim_synblock_get_syn_spell(block) }
}

/// Get the synmaxcol setting from a buffer.
#[must_use]
pub fn buf_synmaxcol(buf: crate::types::BufHandle) -> i32 {
    if buf.is_null() {
        return 0;
    }
    unsafe { nvim_buf_get_synmaxcol(buf) }
}

/// Check if the current syntax state is valid.
#[must_use]
pub fn current_state_valid() -> bool {
    unsafe { nvim_syn_current_state_valid() != 0 }
}

/// Ensure the current syntax state is valid, validating if needed.
pub fn ensure_current_state_valid() {
    unsafe { nvim_syn_ensure_current_state_valid() }
}

/// Get the current line text.
///
/// # Safety
/// The returned pointer is only valid until the next syntax operation.
#[must_use]
pub unsafe fn get_current_line() -> *const std::ffi::c_char {
    nvim_syn_get_current_line()
}

/// Get the attribute for the next match.
#[must_use]
pub fn next_match_attr() -> i32 {
    unsafe { nvim_syn_get_next_match_attr() }
}

/// Get the next match pattern index.
#[must_use]
pub fn next_match_idx() -> i32 {
    unsafe { nvim_syn_get_next_match_idx() }
}

/// Get the next match column.
#[must_use]
pub fn next_match_col() -> i32 {
    unsafe { nvim_syn_get_next_match_col() }
}

/// Check if there is a pending next match.
#[must_use]
pub fn has_next_match() -> bool {
    next_match_idx() >= 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hl_flags_from_raw() {
        let flags = HlFlags::from_raw(HL_ONELINE | HL_KEEPEND);
        assert!(flags.oneline);
        assert!(flags.keepend);
        assert!(!flags.is_match);
        assert!(!flags.conceal);

        let flags = HlFlags::from_raw(HL_CONCEAL | HL_CONCEALENDS);
        assert!(!flags.oneline);
        assert!(flags.conceal);
        assert!(flags.concealends);
    }

    #[test]
    fn test_hl_flags_to_raw() {
        let flags = HlFlags {
            oneline: true,
            keepend: true,
            is_match: false,
            conceal: false,
            concealends: false,
            matchcont: false,
            extend: false,
        };
        assert_eq!(flags.to_raw(), HL_ONELINE | HL_KEEPEND);

        let flags = HlFlags {
            oneline: false,
            keepend: false,
            is_match: true,
            conceal: true,
            concealends: false,
            matchcont: false,
            extend: true,
        };
        assert_eq!(flags.to_raw(), HL_MATCH | HL_CONCEAL | HL_EXTEND);
    }

    #[test]
    fn test_hl_flags_roundtrip() {
        let original = HL_ONELINE | HL_KEEPEND | HL_MATCH | HL_CONCEAL;
        let flags = HlFlags::from_raw(original);
        // Note: Only tests the flags we handle
        assert!(flags.oneline);
        assert!(flags.keepend);
        assert!(flags.is_match);
        assert!(flags.conceal);
    }

    #[test]
    fn test_conceal_result() {
        let not_concealed = ConcealResult::not_concealed();
        assert!(!not_concealed.is_concealed);
        assert_eq!(not_concealed.cchar, 0);
        assert_eq!(not_concealed.seqnr, 0);

        let concealed = ConcealResult::concealed(b'*' as i32, 5);
        assert!(concealed.is_concealed);
        assert_eq!(concealed.cchar, b'*' as i32);
        assert_eq!(concealed.seqnr, 5);
    }

    #[test]
    fn test_resolved_highlight() {
        let hl = ResolvedHighlight::default();
        assert_eq!(hl.attr, 0);
        assert_eq!(hl.syn_id, 0);
        assert_eq!(hl.cchar, 0);
        assert_eq!(hl.flags, 0);
        assert!(!hl.is_concealed());
        assert_eq!(hl.conceal_char(), None);

        let hl = ResolvedHighlight {
            attr: 10,
            syn_id: 5,
            cchar: b'.' as i32,
            flags: HL_CONCEAL,
        };
        assert!(hl.is_concealed());
        assert_eq!(hl.conceal_char(), Some(b'.' as i32));
    }

    #[test]
    fn test_null_handles() {
        let null_block = SynBlockHandle(std::ptr::null_mut());
        let null_pat = SynPatHandle(std::ptr::null_mut());
        let null_item = StateItemHandle(std::ptr::null_mut());
        let null_ke = KeyEntryHandle::null();
        let null_win = WinHandle(std::ptr::null_mut());

        assert!(null_block.is_null());
        assert!(null_pat.is_null());
        assert!(null_item.is_null());
        assert!(null_ke.is_null());
        assert!(null_win.is_null());
    }
}
