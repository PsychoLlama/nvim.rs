//! Syntax buffer attachment.
//!
//! This module handles:
//! - Buffer-specific syntax state initialization
//! - Syntax clearing and reset
//! - Syntax-based folding
//! - Buffer change handlers

use std::ffi::c_int;

use crate::types::{BufHandle, SynBlockHandle, WinHandle};

// =============================================================================
// FFI declarations for buffer operations
// =============================================================================

extern "C" {
    // Buffer syntax state
    fn syntax_start(wp: WinHandle, lnum: c_int);

    // Synblock settings
    fn nvim_synblock_get_syn_error(block: SynBlockHandle) -> c_int;
    fn nvim_synblock_get_syn_slow(block: SynBlockHandle) -> c_int;
    fn nvim_synblock_get_folditems(block: SynBlockHandle) -> c_int;
    fn nvim_synblock_get_syn_foldlevel(block: SynBlockHandle) -> c_int;
    fn nvim_synblock_get_containedin(block: SynBlockHandle) -> c_int;
    fn nvim_synblock_get_conceal(block: SynBlockHandle) -> c_int;

    // Buffer modification tracking
    fn nvim_buf_get_mod_top(buf: BufHandle) -> c_int;
    fn nvim_buf_get_mod_bot(buf: BufHandle) -> c_int;
    fn nvim_buf_get_mod_xlines(buf: BufHandle) -> c_int;

    // Change handling
    fn nvim_syn_stack_apply_changes(buf: BufHandle);

    // Current buffer/window access
    fn nvim_syn_get_buf() -> BufHandle;
    fn nvim_syn_get_block() -> SynBlockHandle;
    fn nvim_syn_get_win() -> WinHandle;

    // Fold level computation
    fn nvim_syn_cur_foldlevel() -> c_int;
}

// =============================================================================
// Buffer syntax initialization
// =============================================================================

/// Start syntax parsing for a line.
///
/// This initializes or restores the syntax state for the given line number,
/// ensuring that highlighting can be computed from that position.
///
/// # Arguments
/// * `wp` - Window handle
/// * `lnum` - Line number (1-based)
///
/// # Safety
/// The window handle must be valid.
pub unsafe fn start_syntax(wp: WinHandle, lnum: i32) {
    if wp.is_null() {
        return;
    }
    syntax_start(wp, lnum);
}

// =============================================================================
// Synblock error and performance state
// =============================================================================

/// Check if syntax highlighting has encountered an error.
#[must_use]
pub fn synblock_has_error(block: SynBlockHandle) -> bool {
    if block.is_null() {
        return false;
    }
    unsafe { nvim_synblock_get_syn_error(block) != 0 }
}

/// Check if syntax highlighting is running slowly.
#[must_use]
pub fn synblock_is_slow(block: SynBlockHandle) -> bool {
    if block.is_null() {
        return false;
    }
    unsafe { nvim_synblock_get_syn_slow(block) != 0 }
}

/// Check if syntax highlighting is enabled and not in error state.
#[must_use]
pub fn synblock_is_active(block: SynBlockHandle) -> bool {
    if block.is_null() {
        return false;
    }
    !synblock_has_error(block) && !synblock_is_slow(block)
}

// =============================================================================
// Synblock fold settings
// =============================================================================

/// Fold level computation mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoldLevelMode {
    /// Use fold level at start of line.
    Start,
    /// Use minimum fold level in line.
    Minimum,
}

/// Fold level mode constants from C.
pub mod foldlevel_mode {
    /// SYNFLD_START - use fold level at start of line.
    pub const START: i32 = 0;
    /// SYNFLD_MINIMUM - use minimum fold level in line.
    pub const MINIMUM: i32 = 1;
}

impl FoldLevelMode {
    /// Convert from raw C value.
    #[must_use]
    pub const fn from_raw(val: i32) -> Self {
        if val == foldlevel_mode::MINIMUM {
            Self::Minimum
        } else {
            Self::Start
        }
    }

    /// Convert to raw C value.
    #[must_use]
    pub const fn to_raw(self) -> i32 {
        match self {
            Self::Start => foldlevel_mode::START,
            Self::Minimum => foldlevel_mode::MINIMUM,
        }
    }
}

/// Get the number of fold items in a synblock.
#[must_use]
pub fn synblock_folditems(block: SynBlockHandle) -> i32 {
    if block.is_null() {
        return 0;
    }
    unsafe { nvim_synblock_get_folditems(block) }
}

/// Check if the synblock has any fold items.
#[must_use]
pub fn synblock_has_folditems(block: SynBlockHandle) -> bool {
    synblock_folditems(block) > 0
}

/// Get the fold level mode for a synblock.
#[must_use]
pub fn synblock_foldlevel_mode(block: SynBlockHandle) -> FoldLevelMode {
    if block.is_null() {
        return FoldLevelMode::Start;
    }
    FoldLevelMode::from_raw(unsafe { nvim_synblock_get_syn_foldlevel(block) })
}

// =============================================================================
// Synblock containedin/conceal settings
// =============================================================================

/// Check if the synblock has any containedin items.
#[must_use]
pub fn synblock_has_containedin(block: SynBlockHandle) -> bool {
    if block.is_null() {
        return false;
    }
    unsafe { nvim_synblock_get_containedin(block) != 0 }
}

/// Check if concealing is enabled for the synblock.
#[must_use]
pub fn synblock_conceal_enabled(block: SynBlockHandle) -> bool {
    if block.is_null() {
        return false;
    }
    unsafe { nvim_synblock_get_conceal(block) != 0 }
}

// =============================================================================
// Buffer modification tracking
// =============================================================================

/// Get the topmost modified line in the buffer.
#[must_use]
pub fn buf_mod_top(buf: BufHandle) -> i32 {
    if buf.is_null() {
        return 0;
    }
    unsafe { nvim_buf_get_mod_top(buf) }
}

/// Get the bottommost modified line in the buffer.
#[must_use]
pub fn buf_mod_bot(buf: BufHandle) -> i32 {
    if buf.is_null() {
        return 0;
    }
    unsafe { nvim_buf_get_mod_bot(buf) }
}

/// Get the number of extra lines added/removed.
#[must_use]
pub fn buf_mod_xlines(buf: BufHandle) -> i32 {
    if buf.is_null() {
        return 0;
    }
    unsafe { nvim_buf_get_mod_xlines(buf) }
}

/// Buffer modification info.
#[derive(Debug, Clone, Copy, Default)]
pub struct ModificationInfo {
    /// Topmost modified line.
    pub top: i32,
    /// Bottommost modified line.
    pub bot: i32,
    /// Number of extra lines (positive = added, negative = removed).
    pub xlines: i32,
}

impl ModificationInfo {
    /// Get modification info for a buffer.
    #[must_use]
    pub fn from_buffer(buf: BufHandle) -> Self {
        if buf.is_null() {
            return Self::default();
        }
        Self {
            top: buf_mod_top(buf),
            bot: buf_mod_bot(buf),
            xlines: buf_mod_xlines(buf),
        }
    }

    /// Check if there are any modifications.
    #[must_use]
    pub const fn has_modifications(&self) -> bool {
        self.top > 0 || self.bot > 0
    }

    /// Get the range of modified lines.
    #[must_use]
    pub const fn modified_range(&self) -> Option<(i32, i32)> {
        if self.has_modifications() {
            Some((self.top, self.bot))
        } else {
            None
        }
    }
}

// =============================================================================
// Change handling
// =============================================================================

/// Apply buffer changes to syntax state.
///
/// This invalidates cached syntax states for modified lines.
///
/// # Safety
/// The buffer handle must be valid.
pub unsafe fn apply_buffer_changes(buf: BufHandle) {
    if buf.is_null() {
        return;
    }
    nvim_syn_stack_apply_changes(buf);
}

// =============================================================================
// Current buffer/window access
// =============================================================================

/// Get the current syntax buffer.
///
/// # Safety
/// Must be called from the main thread during syntax operations.
#[must_use]
pub unsafe fn current_syn_buf() -> BufHandle {
    nvim_syn_get_buf()
}

/// Get the current synblock.
///
/// # Safety
/// Must be called from the main thread during syntax operations.
#[must_use]
pub unsafe fn current_syn_block() -> SynBlockHandle {
    nvim_syn_get_block()
}

/// Get the current syntax window.
///
/// # Safety
/// Must be called from the main thread during syntax operations.
#[must_use]
pub unsafe fn current_syn_win() -> WinHandle {
    nvim_syn_get_win()
}

// =============================================================================
// Fold level computation
// =============================================================================

/// Get the current fold level from the syntax state.
///
/// # Safety
/// Must be called during syntax highlighting.
#[must_use]
pub unsafe fn current_foldlevel() -> i32 {
    nvim_syn_cur_foldlevel()
}

/// Result of fold level computation.
#[derive(Debug, Clone, Copy, Default)]
pub struct FoldLevelResult {
    /// The computed fold level.
    pub level: i32,
    /// Whether the fold level is valid.
    pub valid: bool,
}

impl FoldLevelResult {
    /// Create a valid fold level result.
    #[must_use]
    pub const fn valid(level: i32) -> Self {
        Self { level, valid: true }
    }

    /// Create an invalid fold level result.
    #[must_use]
    pub const fn invalid() -> Self {
        Self {
            level: 0,
            valid: false,
        }
    }
}

// =============================================================================
// Syntax presence check
// =============================================================================

/// Check if syntax highlighting is available and not in error state.
#[must_use]
pub fn can_highlight(block: SynBlockHandle) -> bool {
    if block.is_null() {
        return false;
    }
    synblock_is_active(block)
}

/// Check if syntax-based folding is available.
#[must_use]
pub fn can_compute_folds(block: SynBlockHandle) -> bool {
    can_highlight(block) && synblock_has_folditems(block)
}

// =============================================================================
// FFI exports for buffer integration (Phase Y5)
// =============================================================================

use std::ffi::c_void;

/// Opaque pointer to buffer for FFI
pub type BufPtr = *const c_void;

/// Opaque pointer to synblock for FFI
pub type SynBlockPtr = *const c_void;

/// Buffer syntax status information
#[repr(C)]
pub struct BufferSyntaxStatus {
    /// Whether syntax highlighting is active
    pub active: c_int,
    /// Whether syntax highlighting has errors
    pub has_error: c_int,
    /// Whether syntax highlighting is running slow
    pub is_slow: c_int,
    /// Whether concealing is enabled
    pub conceal_enabled: c_int,
    /// Whether there are containedin items
    pub has_containedin: c_int,
}

/// Get the syntax status for a synblock.
#[no_mangle]
pub unsafe extern "C" fn rs_synblock_status(block: SynBlockPtr) -> BufferSyntaxStatus {
    let handle = SynBlockHandle(block as *mut c_void);
    if handle.is_null() {
        return BufferSyntaxStatus {
            active: 0,
            has_error: 0,
            is_slow: 0,
            conceal_enabled: 0,
            has_containedin: 0,
        };
    }

    let has_error = synblock_has_error(handle);
    let is_slow = synblock_is_slow(handle);

    BufferSyntaxStatus {
        active: if !has_error && !is_slow { 1 } else { 0 },
        has_error: if has_error { 1 } else { 0 },
        is_slow: if is_slow { 1 } else { 0 },
        conceal_enabled: if synblock_conceal_enabled(handle) {
            1
        } else {
            0
        },
        has_containedin: if synblock_has_containedin(handle) {
            1
        } else {
            0
        },
    }
}

/// Check if syntax highlighting can be performed for a synblock.
#[no_mangle]
pub unsafe extern "C" fn rs_synblock_can_highlight(block: SynBlockPtr) -> c_int {
    let handle = SynBlockHandle(block as *mut c_void);
    if can_highlight(handle) {
        1
    } else {
        0
    }
}

/// Check if syntax-based folding can be computed for a synblock.
#[no_mangle]
pub unsafe extern "C" fn rs_synblock_can_fold(block: SynBlockPtr) -> c_int {
    let handle = SynBlockHandle(block as *mut c_void);
    if can_compute_folds(handle) {
        1
    } else {
        0
    }
}

/// Buffer modification info structure for FFI
#[repr(C)]
pub struct BufModInfo {
    /// Topmost modified line (0 if none)
    pub top: c_int,
    /// Bottommost modified line (0 if none)
    pub bot: c_int,
    /// Number of extra lines (positive = added, negative = removed)
    pub xlines: c_int,
    /// Whether there are any modifications
    pub has_mods: c_int,
}

/// Get buffer modification info.
#[no_mangle]
pub unsafe extern "C" fn rs_buf_mod_info(buf: BufPtr) -> BufModInfo {
    let handle = BufHandle(buf as *mut c_void);
    let info = ModificationInfo::from_buffer(handle);

    BufModInfo {
        top: info.top,
        bot: info.bot,
        xlines: info.xlines,
        has_mods: if info.has_modifications() { 1 } else { 0 },
    }
}

/// Apply buffer changes to invalidate syntax state.
#[no_mangle]
pub unsafe extern "C" fn rs_buf_apply_syntax_changes(buf: BufPtr) {
    let handle = BufHandle(buf as *mut c_void);
    apply_buffer_changes(handle);
}

/// Fold level mode constants
#[no_mangle]
pub const extern "C" fn rs_synfld_start() -> c_int {
    foldlevel_mode::START
}

#[no_mangle]
pub const extern "C" fn rs_synfld_minimum() -> c_int {
    foldlevel_mode::MINIMUM
}

/// Get fold level mode for a synblock.
#[no_mangle]
pub unsafe extern "C" fn rs_synblock_get_foldlevel_mode(block: SynBlockPtr) -> c_int {
    let handle = SynBlockHandle(block as *mut c_void);
    synblock_foldlevel_mode(handle).to_raw()
}

/// Check if synblock uses minimum fold level mode.
#[no_mangle]
pub unsafe extern "C" fn rs_synblock_uses_minimum_foldlevel(block: SynBlockPtr) -> c_int {
    let handle = SynBlockHandle(block as *mut c_void);
    if synblock_foldlevel_mode(handle) == FoldLevelMode::Minimum {
        1
    } else {
        0
    }
}

/// Syntax fold info structure
#[repr(C)]
pub struct SynFoldInfo {
    /// Number of fold items in the synblock
    pub fold_items: c_int,
    /// Whether syntax-based folding is available
    pub can_fold: c_int,
    /// The fold level mode (START or MINIMUM)
    pub fold_mode: c_int,
}

/// Get fold information for a synblock.
#[no_mangle]
pub unsafe extern "C" fn rs_synblock_fold_info(block: SynBlockPtr) -> SynFoldInfo {
    let handle = SynBlockHandle(block as *mut c_void);
    if handle.is_null() {
        return SynFoldInfo {
            fold_items: 0,
            can_fold: 0,
            fold_mode: foldlevel_mode::START,
        };
    }

    SynFoldInfo {
        fold_items: synblock_folditems(handle),
        can_fold: if can_compute_folds(handle) { 1 } else { 0 },
        fold_mode: synblock_foldlevel_mode(handle).to_raw(),
    }
}

/// Get the current syntax buffer handle.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_current_buf() -> BufHandle {
    current_syn_buf()
}

/// Get the current synblock handle.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_current_block() -> SynBlockHandle {
    current_syn_block()
}

/// Get the current syntax window handle.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_current_win() -> WinHandle {
    current_syn_win()
}

/// Current syntax context information
#[repr(C)]
pub struct SynContextInfo {
    /// The current buffer handle
    pub buf: BufHandle,
    /// The current synblock handle
    pub block: SynBlockHandle,
    /// The current window handle
    pub win: WinHandle,
    /// Whether syntax highlighting is active
    pub active: c_int,
}

/// Get the current syntax context.
#[no_mangle]
pub unsafe extern "C" fn rs_syn_current_context() -> SynContextInfo {
    let buf = current_syn_buf();
    let block = current_syn_block();
    let win = current_syn_win();
    let active = if !block.is_null() && can_highlight(block) {
        1
    } else {
        0
    };

    SynContextInfo {
        buf,
        block,
        win,
        active,
    }
}

/// Start syntax highlighting at a line.
#[no_mangle]
pub unsafe extern "C" fn rs_syntax_start_at(wp: WinHandle, lnum: c_int) {
    start_syntax(wp, lnum);
}

/// Check if buffer needs syntax state update based on modifications.
#[no_mangle]
pub unsafe extern "C" fn rs_buf_needs_syntax_update(buf: BufPtr) -> c_int {
    let handle = BufHandle(buf as *mut c_void);
    let info = ModificationInfo::from_buffer(handle);
    if info.has_modifications() {
        1
    } else {
        0
    }
}

/// Get the range of lines affected by buffer modifications.
/// Returns 0 for both if no modifications.
#[repr(C)]
pub struct LineRange {
    pub start: c_int,
    pub end: c_int,
}

#[no_mangle]
pub unsafe extern "C" fn rs_buf_modified_range(buf: BufPtr) -> LineRange {
    let handle = BufHandle(buf as *mut c_void);
    let info = ModificationInfo::from_buffer(handle);
    match info.modified_range() {
        Some((start, end)) => LineRange { start, end },
        None => LineRange { start: 0, end: 0 },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_foldlevel_mode() {
        assert_eq!(
            FoldLevelMode::from_raw(foldlevel_mode::START),
            FoldLevelMode::Start
        );
        assert_eq!(
            FoldLevelMode::from_raw(foldlevel_mode::MINIMUM),
            FoldLevelMode::Minimum
        );
        assert_eq!(FoldLevelMode::from_raw(999), FoldLevelMode::Start);

        assert_eq!(FoldLevelMode::Start.to_raw(), foldlevel_mode::START);
        assert_eq!(FoldLevelMode::Minimum.to_raw(), foldlevel_mode::MINIMUM);
    }

    #[test]
    fn test_modification_info() {
        let info = ModificationInfo::default();
        assert!(!info.has_modifications());
        assert_eq!(info.modified_range(), None);

        let info = ModificationInfo {
            top: 10,
            bot: 20,
            xlines: 5,
        };
        assert!(info.has_modifications());
        assert_eq!(info.modified_range(), Some((10, 20)));
    }

    #[test]
    fn test_fold_level_result() {
        let valid = FoldLevelResult::valid(3);
        assert!(valid.valid);
        assert_eq!(valid.level, 3);

        let invalid = FoldLevelResult::invalid();
        assert!(!invalid.valid);
        assert_eq!(invalid.level, 0);
    }

    #[test]
    fn test_null_handles() {
        // Only test is_null() which doesn't call FFI
        let null_buf = BufHandle(std::ptr::null_mut());
        let null_block = SynBlockHandle(std::ptr::null_mut());
        let null_win = WinHandle(std::ptr::null_mut());

        assert!(null_buf.is_null());
        assert!(null_block.is_null());
        assert!(null_win.is_null());

        // Non-null handles
        let non_null_buf = BufHandle(std::ptr::dangling_mut::<std::ffi::c_void>());
        let non_null_block = SynBlockHandle(std::ptr::dangling_mut::<std::ffi::c_void>());

        assert!(!non_null_buf.is_null());
        assert!(!non_null_block.is_null());

        // Note: Cannot call functions like buf_mod_top, synblock_has_error, etc.
        // in tests because they call FFI which isn't available during test linking.
    }
}
