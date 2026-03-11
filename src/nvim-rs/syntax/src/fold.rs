//! Syntax-based folding integration.
//!
//! This module provides utilities for syntax-based folding,
//! tracking fold levels based on syntax regions.

use std::ffi::c_int;

// =============================================================================
// Fold level tracking
// =============================================================================

/// Fold level for a line.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct FoldLevel {
    /// Base fold level.
    pub level: c_int,
    /// Start of fold flag.
    pub start: bool,
    /// End of fold flag.
    pub end: bool,
    /// Nested fold level (for nested regions).
    pub nested: c_int,
}

impl FoldLevel {
    /// Create a fold level.
    #[must_use]
    pub const fn new(level: c_int) -> Self {
        Self {
            level,
            start: false,
            end: false,
            nested: 0,
        }
    }

    /// Create a fold start.
    #[must_use]
    pub const fn start(level: c_int) -> Self {
        Self {
            level,
            start: true,
            end: false,
            nested: 0,
        }
    }

    /// Create a fold end.
    #[must_use]
    pub const fn end(level: c_int) -> Self {
        Self {
            level,
            start: false,
            end: true,
            nested: 0,
        }
    }

    /// Check if this is a fold start.
    #[must_use]
    pub const fn is_start(&self) -> bool {
        self.start
    }

    /// Check if this is a fold end.
    #[must_use]
    pub const fn is_end(&self) -> bool {
        self.end
    }

    /// Check if fold level is zero (no fold).
    #[must_use]
    pub const fn is_zero(&self) -> bool {
        self.level == 0 && self.nested == 0
    }

    /// Get effective level (including nested).
    #[must_use]
    pub const fn effective_level(&self) -> c_int {
        self.level + self.nested
    }
}

// =============================================================================
// Fold state
// =============================================================================

/// State of syntax-based folding for a buffer.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct SynFoldState {
    /// Current fold level.
    pub current_level: c_int,
    /// Maximum fold level seen.
    pub max_level: c_int,
    /// Number of fold regions active.
    pub active_regions: c_int,
    /// Whether fold state is valid.
    pub valid: bool,
    /// Whether we're inside a syntax fold.
    pub in_fold: bool,
}

impl SynFoldState {
    /// Create new fold state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            current_level: 0,
            max_level: 0,
            active_regions: 0,
            valid: true,
            in_fold: false,
        }
    }

    /// Enter a fold region.
    pub fn enter_fold(&mut self) {
        self.current_level += 1;
        self.active_regions += 1;
        self.in_fold = true;
        if self.current_level > self.max_level {
            self.max_level = self.current_level;
        }
    }

    /// Leave a fold region.
    pub fn leave_fold(&mut self) {
        if self.current_level > 0 {
            self.current_level -= 1;
        }
        if self.active_regions > 0 {
            self.active_regions -= 1;
        }
        self.in_fold = self.current_level > 0;
    }

    /// Reset fold state.
    pub fn reset(&mut self) {
        self.current_level = 0;
        self.active_regions = 0;
        self.in_fold = false;
        self.valid = true;
        // Don't reset max_level - that's useful info
    }

    /// Invalidate fold state (needs recomputation).
    pub fn invalidate(&mut self) {
        self.valid = false;
    }

    /// Get current fold level.
    #[must_use]
    pub const fn level(&self) -> c_int {
        self.current_level
    }
}

// =============================================================================
// Fold computation
// =============================================================================

/// Result of computing fold level for a line.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FoldComputeResult {
    /// Fold level of the line.
    pub level: c_int,
    /// Whether level represents fold start.
    pub is_start: bool,
    /// Whether level represents fold end.
    pub is_end: bool,
    /// Flags for folding method.
    pub flags: c_int,
}

impl FoldComputeResult {
    /// Create a result for a regular line.
    #[must_use]
    pub const fn line(level: c_int) -> Self {
        Self {
            level,
            is_start: false,
            is_end: false,
            flags: 0,
        }
    }

    /// Create a result for fold start.
    #[must_use]
    pub const fn start(level: c_int) -> Self {
        Self {
            level,
            is_start: true,
            is_end: false,
            flags: 0,
        }
    }

    /// Create a result for fold end.
    #[must_use]
    pub const fn end(level: c_int) -> Self {
        Self {
            level,
            is_start: false,
            is_end: true,
            flags: 0,
        }
    }

    /// Create undefined result.
    #[must_use]
    pub const fn undefined() -> Self {
        Self {
            level: -1,
            is_start: false,
            is_end: false,
            flags: 0,
        }
    }

    /// Check if result is defined.
    #[must_use]
    pub const fn is_defined(&self) -> bool {
        self.level >= 0
    }
}

impl Default for FoldComputeResult {
    fn default() -> Self {
        Self::line(0)
    }
}

// =============================================================================
// FFI exports
// =============================================================================

/// Create new fold level.
#[no_mangle]
pub extern "C" fn rs_syn_fold_level_new(level: c_int) -> FoldLevel {
    FoldLevel::new(level)
}

/// Create fold level for start.
#[no_mangle]
pub extern "C" fn rs_syn_fold_level_start(level: c_int) -> FoldLevel {
    FoldLevel::start(level)
}

/// Create fold level for end.
#[no_mangle]
pub extern "C" fn rs_syn_fold_level_end(level: c_int) -> FoldLevel {
    FoldLevel::end(level)
}
/// Create new fold state.
#[no_mangle]
pub extern "C" fn rs_syn_fold_state_new() -> SynFoldState {
    SynFoldState::new()
}
/// Create fold compute result for a line.
#[no_mangle]
pub extern "C" fn rs_syn_fold_compute_line(level: c_int) -> FoldComputeResult {
    FoldComputeResult::line(level)
}

/// Create fold compute result for start.
#[no_mangle]
pub extern "C" fn rs_syn_fold_compute_start(level: c_int) -> FoldComputeResult {
    FoldComputeResult::start(level)
}

/// Create fold compute result for end.
#[no_mangle]
pub extern "C" fn rs_syn_fold_compute_end(level: c_int) -> FoldComputeResult {
    FoldComputeResult::end(level)
}
// =============================================================================
// FFI for syn_get_foldlevel implementation
// =============================================================================

use crate::types::{SynBlockHandle, WinHandle};

extern "C" {
    fn nvim_win_get_synblock(wp: WinHandle) -> SynBlockHandle;
    fn nvim_synblock_get_folditems(block: SynBlockHandle) -> c_int;
    fn nvim_synblock_get_syn_error(block: SynBlockHandle) -> c_int;
    fn nvim_synblock_get_syn_slow(block: SynBlockHandle) -> c_int;
    fn nvim_synblock_get_syn_foldlevel(block: SynBlockHandle) -> c_int;
    fn nvim_win_get_foldnestmax(wp: WinHandle) -> c_int;
    fn nvim_syn_is_current_finished() -> c_int;
    fn nvim_syn_get_current_col() -> c_int;
    fn nvim_syn_set_current_col(col: c_int);

    #[link_name = "syntax_start"]
    fn rs_syntax_start(wp: WinHandle, lnum: c_int);
    fn rs_syn_current_attr_impl(
        syncing: c_int,
        displaying: c_int,
        can_spell: *mut c_int,
        keep_state: c_int,
    ) -> c_int;
}

/// SYNFLD_MINIMUM constant
const SYNFLD_MINIMUM: c_int = 1;

/// Real implementation of syn_get_foldlevel.
///
/// # Safety
/// Requires valid window handle.
unsafe fn syn_get_foldlevel_impl(wp: WinHandle, lnum: c_int) -> c_int {
    let mut level = 0;
    let block = nvim_win_get_synblock(wp);

    // Return quickly when there are no fold items at all.
    if nvim_synblock_get_folditems(block) != 0
        && nvim_synblock_get_syn_error(block) == 0
        && nvim_synblock_get_syn_slow(block) == 0
    {
        rs_syntax_start(wp, lnum);

        // Start with the fold level at the start of the line.
        level = crate::state_ops::rs_syn_count_fold_items();

        if nvim_synblock_get_syn_foldlevel(block) == SYNFLD_MINIMUM {
            // Find the lowest fold level that is followed by a higher one.
            let mut cur_level = level;
            let mut low_level = cur_level;
            while nvim_syn_is_current_finished() == 0 {
                rs_syn_current_attr_impl(0, 0, std::ptr::null_mut(), 0);
                cur_level = crate::state_ops::rs_syn_count_fold_items();
                if cur_level < low_level {
                    low_level = cur_level;
                } else if cur_level > low_level {
                    level = low_level;
                }
                let col = nvim_syn_get_current_col();
                nvim_syn_set_current_col(col + 1);
            }
        }
    }

    let fdn = nvim_win_get_foldnestmax(wp);
    if level > fdn {
        level = fdn;
        if level < 0 {
            level = 0;
        }
    }
    level
}

/// Exported entry point for syn_get_foldlevel.
#[export_name = "syn_get_foldlevel"]
pub unsafe extern "C" fn rs_syn_get_foldlevel_impl(wp: WinHandle, lnum: c_int) -> c_int {
    syn_get_foldlevel_impl(wp, lnum)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fold_level() {
        let fl = FoldLevel::new(3);
        assert_eq!(fl.level, 3);
        assert!(!fl.is_start());
        assert!(!fl.is_end());

        let fl = FoldLevel::start(2);
        assert!(fl.is_start());
        assert!(!fl.is_end());

        let fl = FoldLevel::end(2);
        assert!(!fl.is_start());
        assert!(fl.is_end());

        let fl = FoldLevel {
            level: 2,
            nested: 1,
            ..Default::default()
        };
        assert_eq!(fl.effective_level(), 3);
    }

    #[test]
    fn test_fold_state() {
        let mut state = SynFoldState::new();
        assert_eq!(state.level(), 0);
        assert!(!state.in_fold);

        state.enter_fold();
        assert_eq!(state.level(), 1);
        assert!(state.in_fold);

        state.enter_fold();
        assert_eq!(state.level(), 2);
        assert_eq!(state.max_level, 2);

        state.leave_fold();
        assert_eq!(state.level(), 1);
        assert!(state.in_fold);

        state.leave_fold();
        assert_eq!(state.level(), 0);
        assert!(!state.in_fold);
    }

    #[test]
    fn test_fold_compute_result() {
        let r = FoldComputeResult::line(3);
        assert_eq!(r.level, 3);
        assert!(!r.is_start);
        assert!(!r.is_end);
        assert!(r.is_defined());

        let r = FoldComputeResult::start(2);
        assert!(r.is_start);

        let r = FoldComputeResult::end(2);
        assert!(r.is_end);

        let r = FoldComputeResult::undefined();
        assert!(!r.is_defined());
    }
}
