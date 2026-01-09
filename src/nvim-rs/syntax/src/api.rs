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
// C API exports
// =============================================================================

/// Get the highlight attribute for a syntax ID (exported for cbindgen).
#[no_mangle]
pub extern "C" fn rs_syntax_id_to_attr(syn_id: c_int) -> c_int {
    get_syntax_attr(syn_id)
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
