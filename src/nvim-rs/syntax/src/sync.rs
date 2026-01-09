//! Synchronization for syntax highlighting.
//!
//! This module handles:
//! - syn_sync() for scroll/jump synchronization
//! - Sync methods (lines, ccomment, minlines, etc.)
//! - Recovery point detection

use std::ffi::c_int;

use crate::types::{SynBlockHandle, SynPatHandle, SynStateHandle, WinHandle};

// =============================================================================
// Sync flag constants
// =============================================================================

/// Sync flag: use C-style comments for sync.
pub const SF_CCOMMENT: i32 = 0x01;

/// Sync flag: use match patterns for sync.
pub const SF_MATCH: i32 = 0x02;

// =============================================================================
// FFI declarations for sync operations
// =============================================================================

extern "C" {
    // Synblock sync accessors
    fn nvim_synblock_get_sync_flags(block: SynBlockHandle) -> c_int;
    fn nvim_synblock_get_sync_id(block: SynBlockHandle) -> i16;
    fn nvim_synblock_get_sync_minlines(block: SynBlockHandle) -> c_int;
    fn nvim_synblock_get_sync_maxlines(block: SynBlockHandle) -> c_int;
    fn nvim_synblock_get_sync_linebreaks(block: SynBlockHandle) -> c_int;

    // Pattern sync accessors
    fn nvim_synpat_get_syncing(pat: SynPatHandle) -> c_int;
    fn nvim_synpat_get_sync_idx(pat: SynPatHandle) -> c_int;
    fn nvim_synblock_pattern_is_syncing(block: SynBlockHandle, idx: c_int) -> c_int;

    // Current synblock sync accessors
    fn nvim_syn_get_sync_minlines() -> c_int;
    fn nvim_syn_get_sync_maxlines() -> c_int;
    fn nvim_syn_get_sync_flags() -> c_int;
    fn nvim_syn_get_sync_id() -> c_int;

    // Sync highlight accessors
    fn nvim_syn_get_hl_sync_here() -> c_int;
    fn nvim_syn_get_hl_sync_there() -> c_int;

    // Main sync function
    fn nvim_syn_sync(wp: WinHandle, start_lnum: c_int, last_valid: SynStateHandle);

    // Update ends with syncing parameter
    fn nvim_syn_call_syn_update_ends(syncing: c_int);
}

// =============================================================================
// Synblock sync accessors
// =============================================================================

/// Get the sync flags for a synblock.
#[must_use]
pub fn synblock_sync_flags(block: SynBlockHandle) -> i32 {
    if block.is_null() {
        return 0;
    }
    unsafe { nvim_synblock_get_sync_flags(block) }
}

/// Get the sync ID for a synblock.
#[must_use]
pub fn synblock_sync_id(block: SynBlockHandle) -> i16 {
    if block.is_null() {
        return 0;
    }
    unsafe { nvim_synblock_get_sync_id(block) }
}

/// Get the sync minlines for a synblock.
#[must_use]
pub fn synblock_sync_minlines(block: SynBlockHandle) -> i32 {
    if block.is_null() {
        return 0;
    }
    unsafe { nvim_synblock_get_sync_minlines(block) }
}

/// Get the sync maxlines for a synblock.
#[must_use]
pub fn synblock_sync_maxlines(block: SynBlockHandle) -> i32 {
    if block.is_null() {
        return 0;
    }
    unsafe { nvim_synblock_get_sync_maxlines(block) }
}

/// Get the sync linebreaks for a synblock.
#[must_use]
pub fn synblock_sync_linebreaks(block: SynBlockHandle) -> i32 {
    if block.is_null() {
        return 0;
    }
    unsafe { nvim_synblock_get_sync_linebreaks(block) }
}

// =============================================================================
// Pattern sync accessors
// =============================================================================

/// Check if a pattern is a syncing pattern.
#[must_use]
pub fn synpat_is_syncing(pat: SynPatHandle) -> bool {
    if pat.is_null() {
        return false;
    }
    unsafe { nvim_synpat_get_syncing(pat) != 0 }
}

/// Get the sync index for a pattern.
#[must_use]
pub fn synpat_sync_idx(pat: SynPatHandle) -> i32 {
    if pat.is_null() {
        return 0;
    }
    unsafe { nvim_synpat_get_sync_idx(pat) }
}

/// Check if a pattern at a given index is a syncing pattern.
#[must_use]
pub fn synblock_pattern_is_syncing(block: SynBlockHandle, idx: i32) -> bool {
    if block.is_null() {
        return false;
    }
    unsafe { nvim_synblock_pattern_is_syncing(block, idx) != 0 }
}

// =============================================================================
// Current synblock sync accessors
// =============================================================================

/// Get the sync minlines for the current synblock.
#[must_use]
pub fn sync_minlines() -> i32 {
    unsafe { nvim_syn_get_sync_minlines() }
}

/// Get the sync maxlines for the current synblock.
#[must_use]
pub fn sync_maxlines() -> i32 {
    unsafe { nvim_syn_get_sync_maxlines() }
}

/// Get the sync flags for the current synblock.
#[must_use]
pub fn sync_flags() -> i32 {
    unsafe { nvim_syn_get_sync_flags() }
}

/// Get the sync ID for the current synblock.
#[must_use]
pub fn sync_id() -> i32 {
    unsafe { nvim_syn_get_sync_id() }
}

// =============================================================================
// Sync highlight accessors
// =============================================================================

/// Get the HL_SYNC_HERE flag status.
#[must_use]
pub fn hl_sync_here() -> i32 {
    unsafe { nvim_syn_get_hl_sync_here() }
}

/// Get the HL_SYNC_THERE flag status.
#[must_use]
pub fn hl_sync_there() -> i32 {
    unsafe { nvim_syn_get_hl_sync_there() }
}

// =============================================================================
// Main sync function
// =============================================================================

/// Synchronize syntax state for a window at a given line.
///
/// # Safety
/// This modifies global state and must be called from the main thread.
///
/// # Arguments
/// * `wp` - Window handle
/// * `start_lnum` - Line number to sync to
/// * `last_valid` - Last valid synstate (may be null)
pub unsafe fn syn_sync(wp: WinHandle, start_lnum: i32, last_valid: SynStateHandle) {
    nvim_syn_sync(wp, start_lnum, last_valid);
}

/// Call syn_update_ends with syncing parameter.
///
/// # Safety
/// This modifies global state.
pub unsafe fn call_syn_update_ends(syncing: bool) {
    nvim_syn_call_syn_update_ends(if syncing { 1 } else { 0 });
}

// =============================================================================
// Sync settings struct
// =============================================================================

/// Sync settings for a synblock.
#[derive(Debug, Clone, Copy, Default)]
pub struct SyncSettings {
    /// Sync flags (SF_CCOMMENT, SF_MATCH).
    pub flags: i32,
    /// Sync ID.
    pub id: i16,
    /// Minimum lines to search back.
    pub minlines: i32,
    /// Maximum lines to search back.
    pub maxlines: i32,
    /// Line breaks setting.
    pub linebreaks: i32,
}

impl SyncSettings {
    /// Create sync settings from a synblock.
    #[must_use]
    pub fn from_synblock(block: SynBlockHandle) -> Self {
        if block.is_null() {
            return Self::default();
        }
        Self {
            flags: synblock_sync_flags(block),
            id: synblock_sync_id(block),
            minlines: synblock_sync_minlines(block),
            maxlines: synblock_sync_maxlines(block),
            linebreaks: synblock_sync_linebreaks(block),
        }
    }

    /// Create sync settings from the current synblock.
    #[must_use]
    pub fn current() -> Self {
        Self {
            flags: sync_flags(),
            id: sync_id() as i16,
            minlines: sync_minlines(),
            maxlines: sync_maxlines(),
            linebreaks: 0, // Not directly accessible for current
        }
    }

    /// Check if C-style comment sync is enabled.
    #[must_use]
    pub const fn uses_ccomment(&self) -> bool {
        (self.flags & SF_CCOMMENT) != 0
    }

    /// Check if match-based sync is enabled.
    #[must_use]
    pub const fn uses_match(&self) -> bool {
        (self.flags & SF_MATCH) != 0
    }

    /// Check if any sync method is configured.
    #[must_use]
    pub fn has_sync_method(&self) -> bool {
        self.uses_ccomment() || self.uses_match() || self.minlines > 0
    }
}

// =============================================================================
// Sync method enum
// =============================================================================

/// The synchronization method to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncMethod {
    /// No synchronization.
    None,
    /// Synchronize by counting minlines back.
    Minlines,
    /// Synchronize using C-style comments.
    CComment,
    /// Synchronize using match patterns.
    Match,
}

impl SyncMethod {
    /// Determine the sync method from flags.
    #[must_use]
    pub const fn from_flags(flags: i32) -> Self {
        if (flags & SF_CCOMMENT) != 0 {
            Self::CComment
        } else if (flags & SF_MATCH) != 0 {
            Self::Match
        } else {
            Self::None
        }
    }

    /// Determine the sync method from sync settings.
    #[must_use]
    pub fn from_settings(settings: &SyncSettings) -> Self {
        if settings.uses_ccomment() {
            Self::CComment
        } else if settings.uses_match() {
            Self::Match
        } else if settings.minlines > 0 {
            Self::Minlines
        } else {
            Self::None
        }
    }
}

// =============================================================================
// Recovery point helper
// =============================================================================

/// Calculate the line to start searching from for synchronization.
///
/// # Arguments
/// * `start_lnum` - Current line number
/// * `settings` - Sync settings
///
/// # Returns
/// The line number to start searching from.
#[must_use]
pub fn calculate_sync_start(start_lnum: i32, settings: &SyncSettings) -> i32 {
    if settings.minlines >= start_lnum {
        // Start from the beginning
        1
    } else {
        // Calculate based on minlines
        let mut lnum = if settings.minlines == 1 {
            settings.minlines
        } else if settings.minlines < 10 {
            settings.minlines * 2
        } else {
            settings.minlines * 3 / 2
        };

        // Apply maxlines limit
        if settings.maxlines > 0 && lnum > settings.maxlines {
            lnum = settings.maxlines;
        }

        start_lnum.saturating_sub(lnum).max(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_flags() {
        assert_eq!(SF_CCOMMENT, 0x01);
        assert_eq!(SF_MATCH, 0x02);
    }

    #[test]
    fn test_sync_settings_default() {
        let settings = SyncSettings::default();
        assert_eq!(settings.flags, 0);
        assert_eq!(settings.id, 0);
        assert_eq!(settings.minlines, 0);
        assert_eq!(settings.maxlines, 0);
        assert_eq!(settings.linebreaks, 0);
        assert!(!settings.uses_ccomment());
        assert!(!settings.uses_match());
        assert!(!settings.has_sync_method());
    }

    #[test]
    fn test_sync_settings_methods() {
        let settings = SyncSettings {
            flags: SF_CCOMMENT,
            id: 0,
            minlines: 0,
            maxlines: 0,
            linebreaks: 0,
        };
        assert!(settings.uses_ccomment());
        assert!(!settings.uses_match());
        assert!(settings.has_sync_method());

        let settings = SyncSettings {
            flags: SF_MATCH,
            id: 0,
            minlines: 0,
            maxlines: 0,
            linebreaks: 0,
        };
        assert!(!settings.uses_ccomment());
        assert!(settings.uses_match());
        assert!(settings.has_sync_method());

        let settings = SyncSettings {
            flags: 0,
            id: 0,
            minlines: 10,
            maxlines: 0,
            linebreaks: 0,
        };
        assert!(!settings.uses_ccomment());
        assert!(!settings.uses_match());
        assert!(settings.has_sync_method());
    }

    #[test]
    fn test_sync_method() {
        assert_eq!(SyncMethod::from_flags(0), SyncMethod::None);
        assert_eq!(SyncMethod::from_flags(SF_CCOMMENT), SyncMethod::CComment);
        assert_eq!(SyncMethod::from_flags(SF_MATCH), SyncMethod::Match);
        // CComment takes precedence
        assert_eq!(
            SyncMethod::from_flags(SF_CCOMMENT | SF_MATCH),
            SyncMethod::CComment
        );
    }

    #[test]
    fn test_sync_method_from_settings() {
        let settings = SyncSettings::default();
        assert_eq!(SyncMethod::from_settings(&settings), SyncMethod::None);

        let settings = SyncSettings {
            flags: SF_CCOMMENT,
            ..Default::default()
        };
        assert_eq!(SyncMethod::from_settings(&settings), SyncMethod::CComment);

        let settings = SyncSettings {
            flags: SF_MATCH,
            ..Default::default()
        };
        assert_eq!(SyncMethod::from_settings(&settings), SyncMethod::Match);

        let settings = SyncSettings {
            minlines: 10,
            ..Default::default()
        };
        assert_eq!(SyncMethod::from_settings(&settings), SyncMethod::Minlines);
    }

    #[test]
    fn test_calculate_sync_start() {
        // Minlines >= start_lnum
        let settings = SyncSettings {
            minlines: 100,
            maxlines: 0,
            ..Default::default()
        };
        assert_eq!(calculate_sync_start(50, &settings), 1);

        // minlines == 1
        let settings = SyncSettings {
            minlines: 1,
            maxlines: 0,
            ..Default::default()
        };
        assert_eq!(calculate_sync_start(100, &settings), 99);

        // minlines < 10
        let settings = SyncSettings {
            minlines: 5,
            maxlines: 0,
            ..Default::default()
        };
        assert_eq!(calculate_sync_start(100, &settings), 90); // 100 - 10

        // minlines >= 10
        let settings = SyncSettings {
            minlines: 20,
            maxlines: 0,
            ..Default::default()
        };
        assert_eq!(calculate_sync_start(100, &settings), 70); // 100 - 30

        // maxlines limit
        let settings = SyncSettings {
            minlines: 20,
            maxlines: 10,
            ..Default::default()
        };
        assert_eq!(calculate_sync_start(100, &settings), 90); // 100 - 10 (limited)
    }

    #[test]
    fn test_null_handles() {
        let null_block = SynBlockHandle(std::ptr::null_mut());
        let null_pat = SynPatHandle(std::ptr::null_mut());
        let null_state = SynStateHandle(std::ptr::null_mut());
        let null_win = WinHandle(std::ptr::null_mut());

        assert!(null_block.is_null());
        assert!(null_pat.is_null());
        assert!(null_state.is_null());
        assert!(null_win.is_null());
    }
}
