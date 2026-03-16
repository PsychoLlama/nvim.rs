//! Syntax buffer attachment.
//!
//! This module handles:
//! - Buffer-specific syntax state initialization
//! - Syntax clearing and reset
//! - Syntax-based folding
//! - Buffer change handlers

use std::ffi::c_int;

use crate::types::{BufHandle, SynBlockHandle, SynStateHandle, WinHandle};

// =============================================================================
// FFI declarations for buffer operations
// =============================================================================

extern "C" {
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
    #[link_name = "syn_stack_apply_changes"]
    fn nvim_syn_stack_apply_changes(buf: BufHandle);

    // Current buffer/window access
    fn nvim_syn_get_buf() -> BufHandle;
    fn nvim_syn_get_syn_block() -> SynBlockHandle;
    fn nvim_syn_get_win() -> WinHandle;

    // Fold level computation
    #[link_name = "rs_syn_count_fold_items"]
    fn nvim_syn_cur_foldlevel() -> c_int;

    // syntax_start dependencies
    #[link_name = "rs_invalidate_current_state"]
    fn nvim_syn_invalidate_current_state();
    fn nvim_syn_set_syn_buf(buf: BufHandle);
    fn nvim_syn_set_syn_block(block: SynBlockHandle);
    fn nvim_syn_set_syn_win(win: WinHandle);
    fn nvim_syn_buf_get_changed_tick(buf: BufHandle) -> c_int;
    fn nvim_syn_win_get_buffer_ptr(wp: WinHandle) -> BufHandle;
    fn nvim_win_get_synblock(wp: WinHandle) -> SynBlockHandle;
    // (nvim_syn_stack_alloc deleted: call Rust directly)
    fn nvim_synblock_has_sst_array(block: SynBlockHandle) -> c_int;
    fn nvim_syn_set_sst_lasttick(tick: c_int);
    fn nvim_syn_get_display_tick() -> c_int;
    fn nvim_syn_buf_get_line_count(buf: BufHandle) -> c_int;
    #[link_name = "rs_syn_finish_line"]
    fn nvim_syn_finish_line(syncing: c_int) -> c_int;
    fn nvim_syn_get_sync_minlines() -> c_int;
    fn nvim_syn_get_sst_len() -> c_int;
    fn nvim_syn_get_rows() -> c_int;
    #[link_name = "rs_syn_start_line"]
    fn nvim_syn_start_line();
    fn nvim_syn_line_breakcheck();
    fn nvim_syn_get_got_int() -> c_int;
    fn nvim_syn_get_sst_first() -> SynStateHandle;
    // (nvim_syn_stack_find_entry deleted: call Rust directly)
    fn nvim_synstate_get_next(state: SynStateHandle) -> SynStateHandle;
    fn nvim_synstate_get_lnum(state: SynStateHandle) -> c_int;
    fn nvim_synstate_get_change_lnum(state: SynStateHandle) -> c_int;
    fn nvim_synstate_set_change_lnum(p: SynStateHandle, lnum: c_int);

    fn rs_store_current_state() -> SynStateHandle;
    fn rs_load_current_state(from: SynStateHandle);
    fn rs_syn_stack_equal(sp: SynStateHandle) -> c_int;
    fn rs_syn_sync(wp: WinHandle, start_lnum: c_int, last_valid: SynStateHandle);
}

// =============================================================================
// Buffer syntax initialization
// =============================================================================

/// Start syntax parsing for a line.
///
/// Initializes or restores the syntax state for the given line number,
/// ensuring that highlighting can be computed from that position.
///
/// # Safety
/// The window handle must be valid.
pub unsafe fn start_syntax(wp: WinHandle, lnum: i32) {
    if wp.is_null() {
        return;
    }
    syntax_start_impl(wp, lnum);
}

/// Static changedtick to detect buffer modifications between calls.
static mut CHANGEDTICK: c_int = 0;

/// Real implementation of syntax_start, replacing the C version.
///
/// # Safety
/// Requires valid window handle and C global state.
unsafe fn syntax_start_impl(wp: WinHandle, lnum: c_int) {
    let mut last_valid = SynStateHandle::null();
    let mut last_min_valid = SynStateHandle::null();
    let mut prev = SynStateHandle::null();

    crate::statics::CURRENT_SUB_CHAR = 0; // NUL

    // After switching buffers, invalidate current_state.
    let syn_block = nvim_syn_get_syn_block();
    let syn_buf = nvim_syn_get_buf();
    let wp_s = nvim_win_get_synblock(wp);
    let wp_buf = nvim_syn_win_get_buffer_ptr(wp);

    if syn_block.0 != wp_s.0
        || syn_buf.0 != wp_buf.0
        || CHANGEDTICK != nvim_syn_buf_get_changed_tick(wp_buf)
    {
        nvim_syn_invalidate_current_state();
        nvim_syn_set_syn_buf(wp_buf);
        nvim_syn_set_syn_block(wp_s);
    }
    let syn_buf = nvim_syn_get_buf();
    CHANGEDTICK = nvim_syn_buf_get_changed_tick(syn_buf);
    nvim_syn_set_syn_win(wp);

    // Allocate syntax stack when needed.
    crate::cache::rs_syn_stack_alloc();
    let block = nvim_syn_get_syn_block();
    if nvim_synblock_has_sst_array(block) == 0 {
        return; // out of memory
    }
    nvim_syn_set_sst_lasttick(nvim_syn_get_display_tick());

    // If the state of the end of the previous line is useful, store it.
    let current_lnum = crate::statics::CURRENT_LNUM;
    if crate::statics::current_state_is_valid()
        && current_lnum < lnum
        && current_lnum < nvim_syn_buf_get_line_count(syn_buf)
    {
        nvim_syn_finish_line(0);
        if crate::statics::CURRENT_STATE_STORED == 0 {
            crate::statics::CURRENT_LNUM = current_lnum + 1;
            rs_store_current_state();
        }

        // If current_lnum is now the same as "lnum", keep the current state.
        // Otherwise invalidate current_state and figure it out below.
        if crate::statics::CURRENT_LNUM != lnum {
            nvim_syn_invalidate_current_state();
        }
    } else {
        nvim_syn_invalidate_current_state();
    }

    // Try to synchronize from a saved state in b_sst_array[].
    if !crate::statics::current_state_is_valid() && nvim_synblock_has_sst_array(block) != 0 {
        // Find last valid saved state before start_lnum.
        let sync_minlines = nvim_syn_get_sync_minlines();
        let mut p = nvim_syn_get_sst_first();
        while !p.is_null() {
            if nvim_synstate_get_lnum(p) > lnum {
                break;
            }
            if nvim_synstate_get_change_lnum(p) == 0 {
                last_valid = p;
                if nvim_synstate_get_lnum(p) >= lnum - sync_minlines {
                    last_min_valid = p;
                }
            }
            p = nvim_synstate_get_next(p);
        }
        if !last_min_valid.is_null() {
            rs_load_current_state(last_min_valid);
        }
    }

    // If "lnum" is before or far beyond a line with a saved state, need to
    // re-synchronize.
    let first_stored;
    if !crate::statics::current_state_is_valid() {
        rs_syn_sync(wp, lnum, last_valid);
        if crate::statics::CURRENT_LNUM == 1 {
            first_stored = 1;
        } else {
            first_stored = crate::statics::CURRENT_LNUM + nvim_syn_get_sync_minlines();
        }
    } else {
        first_stored = crate::statics::CURRENT_LNUM;
    }

    // Advance from the sync point or saved state until the current line.
    let sst_len = nvim_syn_get_sst_len();
    let rows = nvim_syn_get_rows();
    let dist = if sst_len <= rows {
        999999
    } else {
        nvim_syn_buf_get_line_count(syn_buf) / (sst_len - rows) + 1
    };

    while crate::statics::CURRENT_LNUM < lnum {
        nvim_syn_start_line();
        nvim_syn_finish_line(0);
        let cur_lnum = crate::statics::CURRENT_LNUM;
        crate::statics::CURRENT_LNUM = cur_lnum + 1;

        // If we parsed at least "minlines" lines or started at a valid
        // state, the current state is considered valid.
        let cur_lnum = crate::statics::CURRENT_LNUM;
        if cur_lnum >= first_stored {
            if prev.is_null() {
                prev = crate::cache::rs_syn_stack_find_entry(cur_lnum - 1);
            }

            let mut sp = if prev.is_null() {
                nvim_syn_get_sst_first()
            } else {
                prev
            };
            while !sp.is_null() && nvim_synstate_get_lnum(sp) < cur_lnum {
                sp = nvim_synstate_get_next(sp);
            }

            if !sp.is_null()
                && nvim_synstate_get_lnum(sp) == cur_lnum
                && rs_syn_stack_equal(sp) != 0
            {
                let parsed_lnum = cur_lnum;
                prev = sp;
                while !sp.is_null() && nvim_synstate_get_change_lnum(sp) <= parsed_lnum {
                    if nvim_synstate_get_lnum(sp) <= lnum {
                        prev = sp;
                    } else if nvim_synstate_get_change_lnum(sp) == 0 {
                        break;
                    }
                    nvim_synstate_set_change_lnum(sp, 0);
                    sp = nvim_synstate_get_next(sp);
                }
                rs_load_current_state(prev);
            } else if prev.is_null()
                || cur_lnum == lnum
                || cur_lnum >= nvim_synstate_get_lnum(prev) + dist
            {
                prev = rs_store_current_state();
            }
        }

        // This can take a long time: break when CTRL-C pressed.
        nvim_syn_line_breakcheck();
        if nvim_syn_get_got_int() != 0 {
            crate::statics::CURRENT_LNUM = lnum;
            break;
        }
    }

    nvim_syn_start_line();
}

/// Check if syntax at start of lnum changed since last time.
///
/// This will only be called just after get_syntax_attr() for the previous
/// line, to check if the next line needs to be redrawn too.
///
/// # Safety
/// Requires valid C global state.
unsafe fn syntax_check_changed_impl(lnum: c_int) -> c_int {
    let mut retval: c_int = 1; // true

    // Check the state stack when lnum is just below the previously syntaxed line.
    if crate::statics::current_state_is_valid() && lnum == crate::statics::CURRENT_LNUM + 1 {
        let sp = crate::cache::rs_syn_stack_find_entry(lnum);
        if !sp.is_null() && nvim_synstate_get_lnum(sp) == lnum {
            // finish the previous line (needed when not all of the line was drawn)
            nvim_syn_finish_line(0);

            // Compare the current state with the previously saved state.
            if rs_syn_stack_equal(sp) != 0 {
                retval = 0; // false — no change
            }

            // Store the current state in b_sst_array[] for later use.
            let cur_lnum = crate::statics::CURRENT_LNUM;
            crate::statics::CURRENT_LNUM = cur_lnum + 1;
            rs_store_current_state();
        }
    }

    retval
}

/// End parsing at a given line, updating the saved state entry.
///
/// # Safety
/// Requires valid window handle and C global state.
unsafe fn syntax_end_parsing_impl(wp: WinHandle, lnum: c_int) {
    let block = nvim_syn_get_syn_block();
    let wp_s = nvim_win_get_synblock(wp);
    if block.0 != wp_s.0 {
        return; // not the right window
    }
    let sp = crate::cache::rs_syn_stack_find_entry(lnum);
    if sp.is_null() {
        return;
    }
    let mut target = sp;
    if nvim_synstate_get_lnum(sp) < lnum {
        target = nvim_synstate_get_next(sp);
    }
    if !target.is_null() && nvim_synstate_get_change_lnum(target) != 0 {
        nvim_synstate_set_change_lnum(target, lnum);
    }
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
    nvim_syn_get_syn_block()
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
/// Fold level mode constants
#[no_mangle]
pub const extern "C" fn rs_synfld_start() -> c_int {
    foldlevel_mode::START
}

#[no_mangle]
pub const extern "C" fn rs_synfld_minimum() -> c_int {
    foldlevel_mode::MINIMUM
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
/// Check if syntax at start of lnum changed since last time.
#[no_mangle]
pub unsafe extern "C" fn rs_syntax_check_changed(lnum: c_int) -> c_int {
    syntax_check_changed_impl(lnum)
}

/// C-ABI export: syntax_check_changed returns bool.
#[export_name = "syntax_check_changed"]
pub unsafe extern "C" fn syntax_check_changed_export(lnum: c_int) -> bool {
    syntax_check_changed_impl(lnum) != 0
}

/// End parsing at a given line for a window.
#[export_name = "syntax_end_parsing"]
pub unsafe extern "C" fn rs_syntax_end_parsing_impl(wp: WinHandle, lnum: c_int) {
    syntax_end_parsing_impl(wp, lnum)
}
/// Get the range of lines affected by buffer modifications.
/// Returns 0 for both if no modifications.
#[repr(C)]
pub struct LineRange {
    pub start: c_int,
    pub end: c_int,
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
